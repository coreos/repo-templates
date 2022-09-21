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

use std::fs;
use std::io;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use filetime::{self, FileTime};

use super::schema::*;
use super::*;

const DEFAULT_BRANCH: &str = "DEFAULT";

pub(super) fn update_cache(args: UpdateCacheArgs) -> Result<()> {
    let cfg = Config::parse(&args.config)?;
    do_update_cache(&cfg, &cache_dir(&args.config)?, &args.fork, true)
}

pub(super) fn do_update_cache(
    cfg: &Config,
    cache_dir: &Path,
    fork: &ForkArgs,
    force: bool,
) -> Result<()> {
    for (name, repo) in &cfg.repos {
        // clone repo if missing
        let path = cache_dir.join(name);
        if !path.exists() {
            run_command(
                Command::new("git")
                    .args(["clone", "--depth=1", &repo.url])
                    .arg(&path)
                    .stdout(stderr()?),
            )?;
            // use consistent name for default branch
            run_command(
                Command::new("git")
                    .args(["branch", "-m", DEFAULT_BRANCH])
                    .stdout(stderr()?)
                    .current_dir(&path),
            )?;
        }

        // compute unique identifier of remote branch
        let remote_url = fork
            .regex
            .as_ref()
            .map(|re| re.replace(&repo.url, fork.replacement.as_ref().unwrap()));
        let ident = if let Some(url) = &remote_url {
            format!("{} {}\n", url, fork.branch.as_ref().unwrap())
        } else {
            DEFAULT_BRANCH.into()
        };

        // see if we need to update
        let stamp_path = path.join(".git/tmpl8-stamp");
        // need to switch branches if the stamp contents are different
        match fs::read(&stamp_path) {
            Ok(id) if id == ident.as_bytes() => {
                // update anyway if stale or forced
                let meta = fs::metadata(&stamp_path)
                    .with_context(|| format!("statting {}", stamp_path.display()))?;
                // Update the cache at most once per hour, unless forced
                let age = FileTime::now().seconds()
                    - FileTime::from_last_modification_time(&meta).seconds();
                if (0..3600).contains(&age) && !force {
                    continue;
                }
            }
            Ok(_) => (),
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(e).with_context(|| format!("reading {}", stamp_path.display())),
        }

        // update checkout
        let mut updated = false;
        // remote fork branch exists?
        if let Some(remote_url) = &remote_url {
            if Command::new("git")
                .args(&[
                    "fetch",
                    "--depth",
                    "1",
                    remote_url,
                    fork.branch.as_ref().unwrap(),
                ])
                .stdout(stderr()?)
                .current_dir(&path)
                .status()
                .context("running git fetch")?
                .success()
            {
                run_command(
                    Command::new("git")
                        .args(&["-c", "advice.detachedHead=false", "checkout", "FETCH_HEAD"])
                        .stdout(stderr()?)
                        .current_dir(&path),
                )?;
                updated = true;
            }
        }
        // fall back to default branch
        if !updated {
            run_command(
                Command::new("git")
                    .args(&["checkout", DEFAULT_BRANCH])
                    .stdout(stderr()?)
                    .current_dir(&path),
            )?;
            run_command(
                Command::new("git")
                    .arg("pull")
                    .stdout(stderr()?)
                    .current_dir(&path),
            )?;
        }

        // update stamp
        fs::write(&stamp_path, &ident)
            .with_context(|| format!("writing {}", stamp_path.display()))?;
    }
    Ok(())
}

pub(super) fn cache_dir(config_path: &Path) -> Result<PathBuf> {
    Ok(config_path
        .parent()
        .with_context(|| format!("getting parent of {}", config_path.display()))?
        .join(".cache"))
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

fn stderr() -> Result<Stdio> {
    let stderr_fd = nix::unistd::dup(2_i32.as_raw_fd()).context("duplicating stderr")?;
    Ok(unsafe { Stdio::from_raw_fd(stderr_fd) })
}
