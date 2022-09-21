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
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use regex::Regex;
use similar::TextDiff;
use tera::Tera;
use yansi::Paint;

use super::cache::*;
use super::schema::*;
use super::*;

pub(super) fn render(args: RenderArgs) -> Result<()> {
    let cfg = Config::parse(&args.config)?;
    if let Some(repo) = &args.repo {
        if !cfg.repos.contains_key(repo) {
            bail!("no such repo: {}", repo);
        }
    }

    for (mut path, data) in do_render(&args.config, &cfg)? {
        if let Some(repo) = &args.repo {
            path = match path.strip_prefix(repo) {
                Ok(p) => p.into(),
                Err(_) => continue, // file in another repo
            }
        }
        data.write(&args.output.join(path))?;
    }
    Ok(())
}

pub(super) fn diff(args: DiffArgs) -> Result<()> {
    // render
    let cfg = Config::parse(&args.config)?;
    let rendered = do_render(&args.config, &cfg)?;

    // update Git cache
    let cache_dir = cache_dir(&args.config)?;
    do_update_cache(&cfg, &cache_dir, &args.fork, false)?;

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
        let diff = TextDiff::from_lines(&old_contents, new_contents.as_ref())
            .unified_diff()
            .header(&old_path, &path.to_string_lossy())
            .to_string();
        if diff.is_empty() {
            continue;
        }
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

fn do_render(config_path: &Path, cfg: &Config) -> Result<BTreeMap<PathBuf, RenderedTemplate>> {
    let mut tera = Tera::default();
    tera.add_template_files(
        cfg.templates
            .iter()
            .map(|p| template_path(config_path, p).map(|v| (v, Some(p))))
            .collect::<Result<Vec<_>>>()?,
    )
    .context("parsing templates")?;

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

            let result = RenderedTemplate::new(&tera, template, &ctx)
                .with_context(|| format!("rendering {}", file.path().display()))?;
            if rendered.insert(file.path(), result).is_some() {
                bail!("multiple attempts to write to {}", file.path().display());
            }
        }
    }
    Ok(rendered)
}

struct RenderedTemplate {
    contents: String,
}

impl RenderedTemplate {
    fn new(tera: &Tera, template: &str, ctx: &tera::Context) -> Result<Self> {
        let output = tera.render(template, ctx)?;

        // clean up some common rendering artifacts to ease template writing
        // collapse 3 or more consecutive newlines into 2
        let output = Regex::new("\n{3,}").unwrap().replace_all(&output, "\n\n");
        // collapse 2 or more trailing newlines into 1
        let output = Regex::new("\n{2,}$").unwrap().replace_all(&output, "\n");

        Ok(Self {
            contents: output.to_string(),
        })
    }

    fn write(&self, path: &Path) -> Result<()> {
        let dir = path
            .parent()
            .with_context(|| format!("getting parent of {}", path.display()))?;
        fs::create_dir_all(dir).with_context(|| format!("creating directory {}", dir.display()))?;
        fs::write(&path, &self.contents.as_bytes())
            .with_context(|| format!("writing file {}", path.display()))
    }
}

impl AsRef<String> for RenderedTemplate {
    fn as_ref(&self) -> &String {
        &self.contents
    }
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
