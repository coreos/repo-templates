// Copyright 2022 Red Hat, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use filetime::{self, FileTime};
use regex::Regex;
use similar::TextDiff;
use tera::Tera;
use yansi::Paint;

use super::schema::*;
use super::*;

pub(super) fn render(args: RenderArgs) -> Result<()> {
    let cfg = Config::parse(&args.config)?;
    for (path, data) in do_render(&args.config, &cfg)? {
        let path = args.output.join(path);
        let dir = path
            .parent()
            .with_context(|| format!("getting parent of {}", path.display()))?;
        fs::create_dir_all(dir).with_context(|| format!("creating directory {}", dir.display()))?;
        fs::write(&path, &data.into_bytes())
            .with_context(|| format!("writing file {}", path.display()))?;
    }
    Ok(())
}

pub(super) fn diff(args: DiffArgs) -> Result<()> {
    // render
    let cfg = Config::parse(&args.config)?;
    let rendered = do_render(&args.config, &cfg)?;

    // update Git cache
    let cache_dir = cache_dir(&args.config)?;
    do_update_cache(&cfg, &cache_dir, false)?;

    if args.no_color {
        Paint::disable();
    }
    for (path, new_contents) in &rendered {
        let cache_path = cache_dir.join(path);
        let (old_path, old_contents) = match fs::read_to_string(&cache_path) {
            Ok(c) => (path.to_string_lossy().into_owned(), c),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                ("/dev/null".to_string(), "".to_string())
            }
            Err(e) => {
                return Err(e).with_context(|| format!("reading {}", cache_path.display()))?
            }
        };
        let diff = TextDiff::from_lines(&old_contents, new_contents)
            .unified_diff()
            .header(&old_path, &path.to_string_lossy())
            .to_string();
        for (i, line) in diff.trim_end_matches('\n').split('\n').enumerate() {
            let painted = match line.chars().next() {
                _ if i < 2 => Paint::new(line).bold(),
                Some('-') => Paint::red(line),
                Some('+') => Paint::green(line),
                Some('@') => Paint::cyan(line),
                _ => Paint::new(line),
            };
            println!("{}", painted);
        }
    }

    Ok(())
}

pub(super) fn update_cache(args: UpdateCacheArgs) -> Result<()> {
    let cfg = Config::parse(&args.config)?;
    do_update_cache(&cfg, &cache_dir(&args.config)?, true)
}

fn do_render(config_path: &Path, cfg: &Config) -> Result<BTreeMap<PathBuf, String>> {
    let mut tera = Tera::default();
    tera.add_template_files(
        cfg.templates
            .iter()
            .map(|p| template_path(config_path, p).map(|v| (v, Some(p))))
            .collect::<Result<Vec<_>>>()?,
    )
    .context("parsing templates")?;

    // collapse 3 or more consecutive newlines to ease template writing
    let cleaner = Regex::new("\n{3,}").unwrap();

    let ctx = cfg.vars.to_context()?;
    let mut rendered = BTreeMap::new();
    for template in &cfg.templates {
        let tmpl_cfg = TemplateConfig::parse(&template_config_path(config_path, template)?)?;
        let mut ctx = ctx.clone();
        ctx.extend(tmpl_cfg.vars.to_context()?);

        for file in &tmpl_cfg.files {
            let repo = file.repo(cfg)?;
            let mut ctx = ctx.clone();
            ctx.extend(repo.vars.to_context()?);
            ctx.extend(file.vars.to_context()?);

            let result = tera
                .render(template, &ctx)
                .with_context(|| format!("rendering {}", file.path().display()))?;
            let result = cleaner.replace_all(&result, "\n\n").into_owned();
            if rendered.insert(file.path(), result).is_some() {
                bail!("multiple attempts to write to {}", file.path().display());
            }
        }
    }
    Ok(rendered)
}

fn do_update_cache(cfg: &Config, cache_dir: &Path, force: bool) -> Result<()> {
    for (name, repo) in &cfg.repos {
        let path = cache_dir.join(name);
        let stderr_fd = nix::unistd::dup(2_i32.as_raw_fd()).context("duplicating stderr")?;
        let stderr = unsafe { Stdio::from_raw_fd(stderr_fd) };
        match fs::metadata(&path) {
            Ok(meta) => {
                // Update the cache at most once per hour, unless forced
                let age = FileTime::now().seconds()
                    - FileTime::from_last_modification_time(&meta).seconds();
                if force || !(0..3600).contains(&age) {
                    run_command(
                        Command::new("git")
                            .arg("pull")
                            .stdout(stderr)
                            .current_dir(&path),
                    )?;
                    filetime::set_file_mtime(&path, FileTime::now())
                        .with_context(|| format!("updating timestamp of {}", path.display()))?;
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                run_command(
                    Command::new("git")
                        .args(["clone", "--depth=1", &repo.url])
                        .arg(&path)
                        .stdout(stderr),
                )?;
            }
            Err(e) => return Err(e).with_context(|| format!("querying {}", path.display())),
        }
    }
    Ok(())
}

fn cache_dir(config_path: &Path) -> Result<PathBuf> {
    Ok(config_path
        .parent()
        .with_context(|| format!("getting parent of {}", config_path.display()))?
        .join(".cache"))
}

fn template_path(config_path: &Path, template: &str) -> Result<PathBuf> {
    Ok(config_path
        .parent()
        .with_context(|| format!("path {} has no parent", config_path.display()))?
        .join(template))
}

fn template_config_path(config_path: &Path, template: &str) -> Result<PathBuf> {
    let path = template_path(config_path, template)?;
    let parent = path
        .parent()
        .with_context(|| format!("path {} has no parent", path.display()))?;
    let mut filename = path
        .file_stem()
        .with_context(|| format!("path {} has no filename", path.display()))?
        .to_owned();
    filename.push(".yaml");
    Ok(parent.join(filename))
}

fn run_command(cmd: &mut Command) -> Result<()> {
    let desc = format!(
        "{} {}",
        cmd.get_program().to_string_lossy(),
        cmd.get_args()
            .map(|v| v.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ")
    );
    let status = cmd
        .status()
        .with_context(|| format!("running '{}'", desc))?;
    if !status.success() {
        bail!("command failed: '{}'", desc);
    }
    Ok(())
}
