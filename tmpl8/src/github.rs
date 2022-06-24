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

use std::io;

use anyhow::{Context, Result};
use serde::Serialize;

use super::schema::*;
use super::*;

#[derive(Serialize, Debug)]
struct Matrix {
    repo: Vec<String>,
}

pub(super) fn get_matrix(args: GithubMatrixArgs) -> Result<()> {
    let cfg = Config::parse(&args.config)?;

    const GITHUB_PREFIX: &str = "https://github.com/";
    let matrix = Matrix {
        // ignore non-GitHub repos
        repo: cfg
            .repos
            .iter()
            .map(|(_, r)| &r.url)
            .filter(|u| u.starts_with(GITHUB_PREFIX))
            .map(|u| u.replacen(GITHUB_PREFIX, "", 1))
            .collect(),
    };
    if args.pretty {
        serde_json::to_writer_pretty(&mut io::stdout().lock(), &matrix)
            .context("writing to stdout")?;
    } else {
        print!("::set-output name=matrix::");
        serde_json::to_writer(&mut io::stdout().lock(), &matrix).context("writing to stdout")?;
    }
    println!();

    Ok(())
}
