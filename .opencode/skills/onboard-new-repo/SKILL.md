---
name: onboard-new-repo
description: Add a new downstream repo to config.yaml so it can receive repo-templates
---

# Onboard New Repo

## What it does

1. Validates the repo is not already in `config.yaml`
2. Determines the repo type (Go binary, Rust crate, container, minimal) and required vars
3. Adds the repo entry to `config.yaml` in alphabetical order under `repos:`
4. Optionally enrolls the repo in common templates (delegates to `enroll-repo-in-template`)
5. Validates the rendered output with `make`

## Prerequisites

- The GitHub repo must exist (under the `coreos` org by convention)
- Write access to this repo-templates repository

## Usage

```bash
# Interactive mode
/onboard-new-repo

# Specify repo name and URL
/onboard-new-repo --name my-project --url https://github.com/coreos/my-project

# Specify type for automatic var population
/onboard-new-repo --name my-project --url https://github.com/coreos/my-project --type rust-crate
```

## Workflow

### Step 1: Gather Inputs

If the user did not provide arguments, ask for:

1. **Repo short name** - the key used in `config.yaml` (e.g., `chunkah`, `afterburn`). By convention this matches the GitHub repo name.
2. **Repo URL** - full GitHub URL (e.g., `https://github.com/coreos/chunkah`). Almost always `https://github.com/coreos/<name>`.

### Step 2: Validate Prerequisites

**Check repo is not already registered:**

Read `config.yaml` and verify the repo name does not already appear under `repos:`.

If the repo already exists, inform the user and stop. Suggest using `enroll-repo-in-template` instead if they want to enable templates for it.

### Step 3: Determine Repo Type and Vars

Read `config.yaml` to understand the existing patterns. Ask the user what type of project this is, which determines which `vars` are needed:

**Minimal** (e.g., `chunkah`, `toolbox`, `fedora-coreos-pipeline`):
```yaml
  repo-name:
    url: https://github.com/coreos/repo-name
    vars:
      git_repo: repo-name
```

**Go project with packaging** (e.g., `butane`, `ignition`):
```yaml
  repo-name:
    url: https://github.com/coreos/repo-name
    vars:
      git_repo: repo-name
      fedora_package: repo-name
      pretty_name: Repo Name
      # Optional:
      rhaos_package: repo-name
      rhel9_package: repo-name
      rhel10_package: repo-name
```

**Rust crate** (e.g., `afterburn`, `zincati`):
```yaml
  repo-name:
    url: https://github.com/coreos/repo-name
    vars:
      git_repo: repo-name
      crate: repo-name
      fedora_package: rust-repo-name
      pretty_name: Repo Name
      # Optional:
      library_crate: true          # if it's a library, not a binary
      rhel9_package: rust-repo-name
      rhel10_package: rust-repo-name
```

**Container project** (e.g., `11bot`, `rhcosbot`, `triagebot`):
```yaml
  repo-name:
    url: https://github.com/coreos/repo-name
    vars:
      container_arches: [amd64]    # or [amd64, arm64]
      containers: [quay.io/coreos/repo-name]
      # Plus any of the above vars as needed
```

**Common vars reference** (all optional, add as needed):

| Var | Description | Example |
|-----|-------------|---------|
| `git_repo` | Repo name, used in templates for URLs and paths | `afterburn` |
| `crate` | Rust crate name on crates.io | `afterburn` |
| `library_crate` | Set `true` for library crates (affects release checklist) | `true` |
| `fedora_package` | Fedora package name | `rust-afterburn` |
| `rhaos_package` | RHAOS package name | `butane` |
| `rhel9_package` | RHEL 9 / CentOS Stream 9 package name | `rust-afterburn` |
| `rhel10_package` | RHEL 10 / CentOS Stream 10 package name | `rust-afterburn` |
| `pretty_name` | Human-readable name for release checklists | `Afterburn` |
| `containers` | List of container image URLs | `[quay.io/coreos/afterburn]` |
| `container_arches` | Architectures to build containers for | `[amd64, arm64]` |
| `container_file` | Custom Dockerfile path (default: `Dockerfile`) | `dist/Dockerfile` |
| `container_needs_git_tags` | Whether container build needs full git tags | `true` |
| `quay_repo` | Quay.io repo path for release tagging | `coreos/butane` |
| `quay_legacy_repos` | Legacy Quay repos that also need release tags | `[coreos/fcct]` |

### Step 4: Insert Entry into config.yaml

Add the new repo block to `config.yaml` under `repos:` in **alphabetical order** by repo name.

**Formatting rules:**
- 2-space indent for the repo name key
- 4-space indent for `url:` and `vars:`
- 6-space indent for individual var key-value pairs
- Blank line before the repo block (to separate from the previous entry)
- Repos are sorted alphabetically (e.g., `chunkah` comes after `cap-std-ext`, before `console-login-helper-messages`)

Use the Edit tool to insert the new block at the correct alphabetical position. Find the repo entry that should come immediately AFTER the new one, and insert before it.

### Step 5: Validate

Build and render to verify:

```bash
cd tmpl8 && cargo build
```

```bash
make output
```

Since the new repo has no templates enrolled yet, this should succeed without producing any output files for it. The key thing is that it doesn't break rendering for other repos.

### Step 6: Suggest Next Steps

After onboarding, tell the user:

1. The repo is now registered and can receive templates
2. Suggest common templates to enroll based on the repo type:
   - **All repos**: `dependabot`, `gemini/config`
   - **Go repos**: `go/tests`, `go/release-checklist`, `go/tag_release`, `go/signing-ticket`
   - **Rust repos**: `rust/tests`, `rust/release-checklist`
   - **Container repos**: `container/container`, `container/container-rebuild`
   - **Repos with shell scripts**: `shellcheck`
3. Suggest using the `enroll-repo-in-template` skill to enable templates

## Example Entries by Complexity

### Simplest (3 vars):
```yaml
  chunkah:
    url: https://github.com/coreos/chunkah
    vars:
      git_repo: chunkah
```
*Reference: commit `20419ce`*

### Rust library crate:
```yaml
  ignition-config-rs:
    url: https://github.com/coreos/ignition-config-rs
    vars:
      git_repo: ignition-config-rs
      crate: ignition-config
      library_crate: true
      fedora_package: rust-ignition-config
```

### Full Go project with containers and packaging:
```yaml
  butane:
    url: https://github.com/coreos/ignition
    vars:
      container_needs_git_tags: true
      containers: [quay.io/coreos/butane, quay.io/coreos/fcct]
      git_repo: butane
      repo_subdirectory: butane
      tag_prefix: butane/
      quay_repo: coreos/butane
      quay_legacy_repos: [coreos/fcct]
      fedora_package: butane
      rhaos_package: butane
      rhel9_package: butane
      rhel10_package: butane
      pretty_name: Butane
      vendored_ignition_note: true
```

## Checklist Coverage

- [x] Validating repo doesn't already exist
- [x] Determining correct vars based on project type
- [x] Inserting in alphabetical order
- [x] Matching existing YAML formatting conventions
- [x] Validating render doesn't break

## What's NOT covered

- Creating the actual GitHub repo
- Enrolling in specific templates (use `enroll-repo-in-template` for that)
- Setting up CI/CD in the downstream repo
- Pushing changes or creating PRs

## References

- Example commits:
  - `20419ce` - Add chunkah (minimal repo + gemini enrollment)
  - `99d808f` - Add fedora-coreos-pipeline (minimal repo)
  - `d21ed2f` - Add rhel-coreos-config (minimal repo + shellcheck enrollment)
  - `075b5e5` - Add coreos-assembler & rpm-ostree (two minimal repos at once)
- `config.yaml` - Central registry with all existing repos as reference
- Companion skill: `enroll-repo-in-template` - Enable templates after onboarding
