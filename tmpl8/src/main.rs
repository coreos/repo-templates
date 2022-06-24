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
use clap::Parser;

mod github;
mod render;
mod schema;

/// Renderer for Git repo boilerplate
#[derive(Debug, Parser)]
#[clap(args_conflicts_with_subcommands = true)]
#[clap(disable_help_subcommand = true)]
#[clap(help_expected = true)]
enum Cmd {
    /// Render templates
    Render(RenderArgs),
    /// Print diff from current repo contents
    Diff(DiffArgs),
    /// Render GitHub Actions job matrix
    GithubMatrix(GithubMatrixArgs),
}

#[derive(Debug, Parser)]
struct RenderArgs {
    /// Output directory
    output: PathBuf,
    /// Config file
    #[clap(short = 'c', long, value_name = "file", default_value = "config.yaml")]
    config: PathBuf,
}

#[derive(Debug, Parser)]
struct DiffArgs {
    /// Config file
    #[clap(short = 'c', long, value_name = "file", default_value = "config.yaml")]
    config: PathBuf,
}

#[derive(Debug, Parser)]
struct GithubMatrixArgs {
    /// Config file
    #[clap(short = 'c', long, value_name = "file", default_value = "config.yaml")]
    config: PathBuf,
    /// Print human-readable JSON
    #[clap(short = 'p', long)]
    pretty: bool,
}

fn main() -> Result<()> {
    match Cmd::parse() {
        Cmd::Render(c) => render::render(c),
        Cmd::Diff(c) => render::diff(c),
        Cmd::GithubMatrix(c) => github::get_matrix(c),
    }
}
