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
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub repos: BTreeMap<String, Repo>,
    pub templates: Vec<String>,
    #[serde(default)]
    pub vars: Vars,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Repo {
    pub url: String,
    // overrides TemplateConfig.vars
    #[serde(default)]
    pub vars: Vars,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TemplateConfig {
    pub files: Vec<File>,
    // overrides Config.vars
    #[serde(default)]
    pub vars: Vars,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct File {
    pub repo: String,
    pub path: String,
    // overrides Repo.vars
    #[serde(default)]
    pub vars: Vars,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Vars {
    #[serde(flatten)]
    vars: BTreeMap<String, serde_yaml::Value>,
}

pub trait Parseable: Sized
where
    Self: DeserializeOwned,
{
    /// Parse struct from a YAML file
    fn parse(path: &Path) -> Result<Self> {
        let f = OpenOptions::new()
            .read(true)
            .open(path)
            .with_context(|| format!("opening {}", path.display()))?;
        serde_yaml::from_reader(f).with_context(|| format!("parsing {}", path.display()))
    }
}

impl Parseable for Config {}
impl Parseable for TemplateConfig {}

impl File {
    /// Look up Repo from Config
    pub fn repo<'a>(&self, cfg: &'a Config) -> Result<&'a Repo> {
        cfg.repos
            .get(&self.repo)
            .with_context(|| format!("no such repo: {}", self.repo))
    }

    pub fn path(&self) -> PathBuf {
        let mut ret = PathBuf::from(&self.repo);
        ret.push(&self.path);
        ret
    }
}

impl Vars {
    /// Convert to Tera context
    pub fn to_context(&self) -> Result<tera::Context> {
        Ok(tera::Context::from_serialize(&self.vars)?)
    }
}
