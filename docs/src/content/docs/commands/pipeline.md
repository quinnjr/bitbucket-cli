---
title: pipeline
description: Trigger, monitor, and manage CI/CD pipelines and their configuration.
sidebar:
  order: 4
---

The `bitbucket pipeline` command group drives Bitbucket Pipelines end to end — listing and inspecting runs, triggering new ones (optionally waiting for completion), stopping and re-running builds, and managing the surrounding configuration: repository and workspace variables, schedules, and dependency caches. Most subcommands take the repository as `workspace/repo-slug`; the workspace can be omitted when a default is configured with `--workspace` or in your config. Every command also accepts `--output table` (default) or `--output json` for scripting.

## Subcommands

| Command | Description |
| --- | --- |
| `pipeline list` | List pipelines |
| `pipeline view` | View pipeline details |
| `pipeline trigger` | Trigger a new pipeline |
| `pipeline stop` | Stop a running pipeline |
| `pipeline rerun` | Re-run a pipeline by triggering an equivalent one |
| `pipeline config` | View or update the repository pipelines configuration |
| `pipeline variable` | Manage repository pipeline variables |
| `pipeline workspace-variable` | Manage workspace-level pipeline variables |
| `pipeline schedule` | Manage pipeline schedules |
| `pipeline cache` | Manage pipeline dependency caches |

The `list` command supports `--limit`, `--status`, `--target-branch`, `--sort`, and `--page` for filtering and paging. `view` requires `--build` and can show logs with `--logs`, a single step with `--step`, or complete logs with `--full-logs`. `trigger` runs a branch (`--branch`, default `main`), a specific `--commit`, or a custom `--pipeline`, passing `--var KEY=VALUE` and `--secured-var KEY=VALUE`, and can block with `--wait`. `stop` targets a build by `--build` or `--uuid`; `rerun` re-runs an existing `--build`. `config` toggles pipelines with `--enable`/`--disable` or sets `--next-build-number`.

### `pipeline variable`

Manage repository-scoped pipeline variables.

| Command | Description |
| --- | --- |
| `pipeline variable list` | List pipeline variables |
| `pipeline variable set` | Create or update a pipeline variable (`--secured` to mask the value) |
| `pipeline variable delete` | Delete a pipeline variable by key (`-y`/`--yes` to skip the prompt) |

`variable set` takes positional `<REPO> <KEY> <VALUE>`, and `variable delete` takes `<REPO> <KEY>`.

### `pipeline workspace-variable`

Manage variables shared across every repository in a workspace. The workspace is an optional positional (`[WORKSPACE]`) that falls back to `--workspace` or the configured default.

| Command | Description |
| --- | --- |
| `pipeline workspace-variable list` | List workspace pipeline variables |
| `pipeline workspace-variable set` | Create or update a workspace pipeline variable (`--secured` to mask the value) |
| `pipeline workspace-variable delete` | Delete a workspace pipeline variable by key (`-y`/`--yes` to skip the prompt) |

`workspace-variable set` takes positional `<KEY> <VALUE> [WORKSPACE]`, and `workspace-variable delete` takes `<KEY> [WORKSPACE]`.

### `pipeline schedule`

Manage recurring pipeline runs.

| Command | Description |
| --- | --- |
| `pipeline schedule list` | List pipeline schedules |
| `pipeline schedule create` | Create a pipeline schedule (requires `--branch` and `--cron`) |
| `pipeline schedule delete` | Delete a pipeline schedule by UUID (`-y`/`--yes` to skip the prompt) |

`schedule create` takes positional `<REPO>`, and `schedule delete` takes `<REPO> <UUID>` (UUID with or without braces).

### `pipeline cache`

Manage the dependency caches Pipelines builds up between runs.

| Command | Description |
| --- | --- |
| `pipeline cache list` | List pipeline dependency caches |
| `pipeline cache delete` | Delete pipeline dependency caches by name (`-y`/`--yes` to skip the prompt) |

`cache delete` takes positional `<REPO> <NAME>`, using the name shown by `cache list`.

## Examples

List the most recent pipelines for a repository, filtering by status:

```bash
bitbucket pipeline list myworkspace/myrepo --status IN_PROGRESS
```

Trigger a new pipeline on a branch, passing build variables:

```bash
bitbucket pipeline trigger myworkspace/myrepo \
  --branch release/2.0 \
  --var DEPLOY_ENV=staging \
  --var REGION=us-east-1
```

Inspect a finished build, printing the full logs for a single step:

```bash
bitbucket pipeline view myworkspace/myrepo --build 128 --step 2 --full-logs
```

Set a repository pipeline variable, masking its value in logs and the UI:

```bash
bitbucket pipeline variable set myworkspace/myrepo API_TOKEN s3cr3t-value --secured
```

Create a schedule that runs a branch on a cron pattern:

```bash
bitbucket pipeline schedule create myworkspace/myrepo \
  --branch main \
  --cron "0 0 12 * * ? *"
```

Re-run an existing pipeline by its build number:

```bash
bitbucket pipeline rerun myworkspace/myrepo --build 128
```

Stop a running pipeline by build number:

```bash
bitbucket pipeline stop myworkspace/myrepo --build 129
```
