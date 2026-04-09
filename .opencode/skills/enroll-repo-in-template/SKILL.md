---
name: enroll-repo-in-template
description: Enable an existing Tera template for a downstream repo in the repo-templates system
---

# Enroll Repo in Template

## What it does

1. Validates the repo exists in `config.yaml`
2. Identifies all YAML config files for the chosen template
3. Reads template defaults and existing entries to determine the pattern
4. Adds the repo entry to each config YAML file in alphabetical order
5. Applies any user-specified variable overrides
6. Validates the rendered output with `make`

## Prerequisites

- The repo must already be registered in `config.yaml` under `repos:`
- The template must already exist and be listed in `config.yaml` under `templates:`
- Cargo must be available to build the `tmpl8` renderer (for validation)

## Usage

```bash
# Interactive mode - will ask which template and repo
/enroll-repo-in-template

# Specify template and repo
/enroll-repo-in-template --template shellcheck --repo coreos-assembler

# With variable overrides
/enroll-repo-in-template --template shellcheck --repo coreos-assembler --vars 'branches: [main, rhel-*]'
```

## Workflow

### Step 1: Gather Inputs

If the user did not provide arguments, determine the template and repo interactively.

**Get the template name:**

Read `config.yaml` and extract the `templates:` list to show available templates:

```bash
grep -A 100 '^templates:' config.yaml
```

Ask the user which template they want to enable if not specified.

**Get the repo name:**

Read `config.yaml` and extract the `repos:` list to show available repos:

```bash
grep -E '^\s{2}\w' config.yaml | sed 's/://g' | awk '{print $1}'
```

Ask the user which repo they want to enroll if not specified.

### Step 2: Validate Prerequisites

**Check repo exists in `config.yaml`:**

```bash
grep -q "^  REPO_NAME:" config.yaml
```

If the repo does NOT exist, stop and tell the user to add it first (or suggest running the "onboard-repo" skill if it exists).

**Check template is registered:**

Verify the template appears in the `templates:` list in `config.yaml`.

**Check repo is not already enrolled:**

For each template config YAML file, check that the repo is not already listed:

```bash
grep -q "repo: REPO_NAME" TEMPLATE_DIR/CONFIG.yaml
```

If already enrolled, inform the user and stop.

### Step 3: Identify Template Config Files

Templates may have one or more YAML config files. Determine which ones exist:

```bash
ls TEMPLATE_DIR/*.yaml
```

For example:
- `shellcheck` has `script.yaml` AND `workflow.yaml` (both need entries)
- `gemini` has just `config.yaml` (one entry needed)
- `dependabot` has just `dependabot.yaml` (one entry needed)

Read each YAML config file to understand:
1. The default `vars:` at the top of the file
2. The `path:` pattern used by existing entries
3. Whether entries typically include `vars:` overrides

### Step 4: Determine Entry Details

For each config YAML file:

1. **Determine the `path:`** - Look at existing entries. If all repos use the same path, use that. If paths vary, ask the user.

2. **Determine `vars:` overrides** - Read the template-level `vars:` defaults. Ask the user if they need to override any variables. Present the available variables and their defaults.

3. **Find insertion point** - Entries in the `files:` list should be in alphabetical order by repo name. Find the correct position.

### Step 5: Edit the Config Files

For each template config YAML file, use the Edit tool to insert the new entry.

**Format for entry WITHOUT vars overrides:**
```yaml

  - repo: REPO_NAME
    path: OUTPUT_PATH
```

**Format for entry WITH vars overrides:**
```yaml

  - repo: REPO_NAME
    path: OUTPUT_PATH
    vars:
      VAR_NAME: VAR_VALUE
```

**Important formatting rules:**
- Entries are separated by blank lines
- Use 2-space indentation for the `- repo:` line (under `files:`)
- Use 4-space indentation for `path:` and `vars:`
- Use 6-space indentation for individual var key-value pairs
- Maintain alphabetical order by repo name within the `files:` list
- Match the exact indentation style used by surrounding entries

### Step 6: Validate

After making changes, validate the template renders correctly:

```bash
# Build the renderer if not already built
cd tmpl8 && cargo build

# Render all templates and show diffs
make diff
```

Or just render the output tree:

```bash
make output
```

Check that:
1. No rendering errors occurred
2. The new repo's rendered file appears in `output/REPO_NAME/OUTPUT_PATH`
3. The rendered content looks correct (template variables substituted properly)

### Step 7: Report Results

Tell the user:
- Which files were modified
- What the rendered output looks like
- Remind them to commit, PR, and let CI validate

## Template Quick Reference

### Templates with a single config file (simple):
- `container/container.yaml` -> `.github/workflows/container.yml`
- `container/container-rebuild.yaml` -> `.github/workflows/container-rebuild.yml`
- `copr/Makefile.yaml` -> `Makefile`
- `dependabot/dependabot.yaml` -> `.github/dependabot.yml`
- `fcos/release-checklist.yaml` -> `.github/ISSUE_TEMPLATE/{stream}.md`
- `gemini/config.yaml` -> `.gemini/config.yaml`
- `go/release-checklist.yaml` -> `.github/ISSUE_TEMPLATE/release-checklist.md`
- `go/signing-ticket.yaml` -> `signing-ticket.sh`
- `go/tag_release.yaml` -> `tag_release.sh`
- `go/tests.yaml` -> `.github/workflows/go.yml`
- `go/tests-ignition.yaml` -> `.github/workflows/go.yml`
- `owners-file-action/owners-file-action.yaml` -> `.github/workflows/owners-file-action.yml`
- `release-notes/require-release-note.yaml` -> `.github/workflows/require-release-note.yml`
- `rust/release-checklist.yaml` -> `.github/ISSUE_TEMPLATE/release-checklist.md`
- `rust/rpm-test.yaml` -> `.github/workflows/rpm-test.yml`
- `rust/tests.yaml` -> `.github/workflows/rust.yml`

### Templates with multiple config files (need entries in ALL):
- `shellcheck/` -> `script.yaml` (path: `ci/shellcheck`) + `workflow.yaml` (path: `.github/workflows/shellcheck.yml`)
- `find-whitespace/` -> `script.yaml` (path: `ci/find-whitespace`) + `workflow.yaml` (path: `.github/workflows/find-whitespace.yml`)
- `docs/` -> `coreos.yaml` covers both `docs/_config.yml` and `docs/assets/css/coreos.scss`

## Checklist Coverage

This skill automates the following manual steps:

- [x] Finding the correct template config YAML file(s)
- [x] Understanding available variables and defaults
- [x] Determining the correct output path
- [x] Adding the entry in alphabetical order
- [x] Handling multi-file templates consistently
- [x] Validating the rendered output

## What's NOT covered

- Adding a new repo to `config.yaml` (use the "onboard-repo" skill or do manually)
- Creating a brand new template type (different workflow)
- Modifying template content (the `.yml` Tera template files themselves)
- Pushing changes to remote or creating PRs

## Example Output

When you run `/enroll-repo-in-template --template gemini/config --repo fedora-coreos-pipeline`:

```
Validating prerequisites...
  Repo 'fedora-coreos-pipeline' exists in config.yaml
  Template 'gemini/config.yml' is registered in config.yaml
  Repo is not already enrolled in gemini/config

Reading template config: gemini/config.yaml
  Default vars: branches: [main]
  Common path: .gemini/config.yaml
  Config files to update: 1

Adding entry to gemini/config.yaml...
  Inserted after 'fedora-coreos-config', before 'ignition'

Validating render...
  make output completed successfully
  Output file: output/fedora-coreos-pipeline/.gemini/config.yaml

Done! Files modified:
  - gemini/config.yaml (+3 lines)

Next steps:
  1. Review the changes with 'git diff'
  2. Commit and create a PR
  3. CI will validate rendering and show diffs against downstream repos
```

## References

- [DESIGN.md](DESIGN.md) - Detailed design document with analysis
- [Example 1](examples/example-1-simple-enrollment.md) - Simple single-file enrollment
- [Example 2](examples/example-2-multi-file-with-vars.md) - Multi-file enrollment with vars
- [Example 3](examples/example-3-enrollment-with-custom-vars.md) - Enrollment with custom vars
- `config.yaml` - Central registry of repos and templates
- `README.md` - Template system architecture documentation
