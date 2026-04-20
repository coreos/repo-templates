# CoreOS repo-templates

Centralized Tera template management for 40+ CoreOS GitHub repositories. Renders parameterized CI/CD workflows, release checklists, and configs, then auto-PRs changes to downstream repos.

## Tech Stack

- **Language**: Rust (tmpl8 renderer), Tera templates, Bash, YAML
- **Template Engine**: [Tera](https://tera.netlify.app/docs/#templates) (Jinja2-like)
- **Build**: Cargo (Rust), Make
- **CI/CD**: GitHub Actions (Fedora container)
- **Testing**: `make diff` (renders and diffs against downstream repos)

## Architecture

```
config.yaml              # Central config: repos, templates, global vars
<template-dir>/
  template.yml            # Tera template (source of truth)
  template.yaml           # Metadata: which repos get this template + vars
tmpl8/                    # Rust CLI tool that renders templates
  src/main.rs             # Entry point (clap CLI)
  src/schema.rs           # YAML config schema
  src/render.rs           # Rendering + diff logic
  src/cache.rs            # Git repo cache management
  src/github.rs           # GitHub Actions matrix generation
.github/workflows/
  check.yml               # PR validation: render diffs
  sync.yml                # Post-merge: auto-PR to downstream repos
```

**Key pattern**: Each template directory has `.yml` (Tera template) + `.yaml` (metadata) pairs. The `.yaml` lists `(repo, output_path)` tuples and context variable overrides.

**Variable precedence** (highest to lowest): file-specific vars in template YAML > repo-specific vars in `config.yaml` > global vars in template YAML > global vars in `config.yaml`.

## Build Commands

- `make tmpl8` - Build the Rust renderer (`cd tmpl8 && cargo build`)
- `make diff` - Render all templates and diff against downstream repos
- `make output` - Render all templates to `output/` directory
- `make sync` - Force sync downstream repo cache

## Workflow

1. Edit templates locally
2. Run `make diff` to verify rendered output
3. PR changes -- reviewers check "Render diffs" CI step
4. Merge -- CI auto-submits PRs to affected downstream repos

## Code Style

- YAML: 2-space indentation
- Rust: standard `rustfmt` conventions, Edition 2021
- Shell scripts: `set -euo pipefail`, function-based structure
- All templates include header: `# Maintained in https://github.com/coreos/repo-templates`
- Repo entries in `config.yaml` and template `.yaml` files are alphabetically ordered

## Commit Conventions

**Format**: `scope: description` (lowercase, imperative)

Examples from history:
- `go/tests: add ability to opt out of running go test`
- `dependabot: remove github-actions for downstream updates`
- `rust: Bump lint toolchain to 1.90.0`
- `config: add fedora-coreos-pipeline`
- `release-checklist: fix releng ticket link`

**Scope**: template directory name, `config`, or descriptive component. No file extensions in scope.

## Important Rules

- **Template files are the source of truth** -- the `.yml` files here are edited directly; "do not edit" headers are for downstream copies only
- **Do not edit downstream repos directly** for templated files
- `config.yaml` repo entries must be in **alphabetical order**
- Template `.yaml` file entries must also be in **alphabetical order** by repo name
- `Cargo.lock` is auto-generated -- do not edit manually
- `.cache/` and `output/` directories are gitignored; never commit them

## OpenCode Skills

Two skills are available in `.opencode/skills/`:
- **onboard-new-repo**: Add a new downstream repo to `config.yaml`
- **enroll-repo-in-template**: Enable a template for an existing downstream repo
