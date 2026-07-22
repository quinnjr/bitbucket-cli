---
title: workspace
description: List workspaces, members, permissions, and projects.
sidebar:
  order: 5
---

The `bitbucket workspace` command group manages workspaces — list the ones you can access, inspect a workspace's details, members, and permissions, and manage the projects inside it.

## Subcommands

| Command | Description |
| --- | --- |
| `workspace list` | List workspaces you have access to |
| `workspace view` | View workspace details |
| `workspace members` | List members of a workspace |
| `workspace permissions` | List user permissions on a workspace or one of its repositories |
| `workspace project` | Manage projects in a workspace |

`workspace list` supports `--role <member\|collaborator\|owner>`, `--query` (BBQL), `--sort`, `--limit`, and `--all`. The `view`, `members`, and `permissions` commands take an optional `[workspace]` slug that falls back to `--workspace` or the configured default workspace. `workspace permissions` accepts `--repo <repo>` to show permissions on a repository instead of the workspace.

### `workspace project`

`workspace project` manages the projects inside a workspace.

| Command | Description |
| --- | --- |
| `workspace project list` | List projects in a workspace |
| `workspace project view` | View project details |
| `workspace project create` | Create a new project |
| `workspace project edit` | Update project settings |
| `workspace project delete` | Delete a project |

`project create` takes a `<key>` (e.g. `PROJ`) and `<name>`, plus `--description`, and `--private` (the default) or `--public`. `project edit` accepts `--name`, `--description`, `--private`, and `--public`. `project delete` accepts `-y`/`--yes` to skip the confirmation prompt.

## Examples

List the workspaces you own:

```bash
bitbucket workspace list --role owner
```

View a workspace and its members:

```bash
bitbucket workspace view myworkspace
bitbucket workspace members myworkspace
```

Check your permissions on a specific repository:

```bash
bitbucket workspace permissions myworkspace --repo myrepo
```

List and create projects:

```bash
bitbucket workspace project list myworkspace --sort name
bitbucket workspace project create PROJ "Platform" --description "Core services" --private
```

Delete a project without the confirmation prompt:

```bash
bitbucket workspace project delete PROJ --yes
```
