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

use std::path::PathBuf;

use anyhow::Result;
use clap::builder::ArgPredicate;
use clap::Parser;
use regex::Regex;

mod cache;
mod github;
mod render;
mod schema;

/// Renderer for Git repo boilerplate
#[derive(Debug, Parser)]
#[command(args_conflicts_with_subcommands = true)]
#[command(disable_help_subcommand = true)]
#[command(help_expected = true)]
enum Cmd {
    /// Render templates
    Render(RenderArgs),
    /// Print diff from current repo contents
    Diff(DiffArgs),
    /// Update cache for diff command (usually unnecessary)
    UpdateCache(UpdateCacheArgs),
    /// Render GitHub Actions job matrix
    GithubMatrix(GithubMatrixArgs),
}

#[derive(Debug, Parser)]
struct RenderArgs {
    /// Output directory
    output: PathBuf,
    /// Config file
    #[arg(short = 'c', long, value_name = "file", default_value = "config.yaml")]
    config: PathBuf,
    /// Render only one repository
    #[arg(short = 'r', long, value_name = "repo-name")]
    repo: Option<String>,
}

#[derive(Debug, Parser)]
struct DiffArgs {
    /// Config file
    #[arg(short = 'c', long, value_name = "file", default_value = "config.yaml")]
    config: PathBuf,
    #[command(flatten)]
    fork: ForkArgs,
    /// Disable color output
    #[arg(short = 'n', long)]
    no_color: bool,
}

#[derive(Debug, Parser)]
struct UpdateCacheArgs {
    /// Config file
    #[arg(short = 'c', long, value_name = "file", default_value = "config.yaml")]
    config: PathBuf,
    #[command(flatten)]
    fork: ForkArgs,
}

#[derive(Debug, Parser)]
struct ForkArgs {
    /// Regex for the upstream part of repo URL
    #[arg(long = "fork-regex", value_name = "regex")]
    #[arg(requires_ifs = [
        (ArgPredicate::IsPresent, "replacement"),
        (ArgPredicate::IsPresent, "branch")
    ])]
    regex: Option<Regex>,
    /// Replacement for upstream part of repo URL
    #[arg(long = "fork-replacement", value_name = "string")]
    #[arg(requires_ifs = [
        (ArgPredicate::IsPresent, "regex"),
        (ArgPredicate::IsPresent, "branch")
    ])]
    replacement: Option<String>,
    /// Fork branch
    #[arg(long = "fork-branch", value_name = "branch")]
    #[arg(requires_ifs = [
        (ArgPredicate::IsPresent, "regex"),
        (ArgPredicate::IsPresent, "replacement")
    ])]
    branch: Option<String>,
}

#[derive(Debug, Parser)]
struct GithubMatrixArgs {
    /// Config file
    #[arg(short = 'c', long, value_name = "file", default_value = "config.yaml")]
    config: PathBuf,
    /// Print human-readable JSON
    #[arg(short = 'p', long)]
    pretty: bool,
}

fn main() -> Result<()> {
    match Cmd::parse() {
        Cmd::Render(c) => render::render(c),
        Cmd::Diff(c) => render::diff(c),
        Cmd::UpdateCache(c) => cache::update_cache(c),
        Cmd::GithubMatrix(c) => github::get_matrix(c),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn clap_app() {
        Cmd::command().debug_assert()
    }
}
