# CoreOS repo templates

This repository includes parameterized templates for release checklists
used by many different CoreOS repositories.

## Structure of this repo

This repo contains [rendering code](tmpl8), some GitHub Actions workflows,
and the [Tera templates](https://tera.netlify.app/docs/#templates) and
metadata that are rendered and submitted to downstream Git repos.  Each
Tera template has a adjacent `.yaml` file describing how the template will
be applied, and there is also a top-level `config.yaml` tying everything
together.

Tera templates are rendered with a "context", which is a set of key-value
pairs that can be textually substituted into the template and can also
affect control flow when rendering it.  Context variables are provided via
the YAML files.

`config.yaml` contains a list of downstream Git repos that we manage and a
list of templates that we render.  Each Git repo is associated with a short
name, which the GitHub Actions workflow expects to match the repo name.
`config.yaml` also defines global and repo-specific context variables.

Template YAML files contain a list of files, as `(repo, path)` tuples, to be
derived from the corresponding template.  Repos are referenced by their name
in the `config.yaml` repo list.  The template YAML also defines
template-specific and file-specific context variables.

The sources of context variables, from highest to lowest precedence, are:

- File-specific `vars` in template YAML
- Repo-specific `vars` in `config.yaml`
- Global `vars` in template YAML
- Global `vars` in `config.yaml`

## Modifying templates

To modify templated artifacts:

1. Clone this repo and make your changes locally.  Run `make` to compare
the resulting rendered files to the versions currently stored in the
downstream repositories, or `make output` to generate a complete rendered
tree for examination.

2. PR your changes.  Reviewers can view the "Render diffs" step of the
"Render" CI job to see the changes that will be PRed to the various repos.

3. Merge your changes.  CI will automatically submit PRs to affected
downstream repos.
