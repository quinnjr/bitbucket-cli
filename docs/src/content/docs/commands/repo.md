---
title: repo
description: Manage repositories, branches, tags, commits, files, webhooks, permissions, and more.
sidebar:
  order: 1
---

The `bitbucket repo` command group manages repositories and everything scoped to them: branches, tags, commits, files, downloads, webhooks, permissions, deploy keys, environments, deployments, and the branching model.

Every subcommand accepts `--workspace <WORKSPACE>` to supply the workspace when it is omitted from arguments, and `--output <table|json>` to choose the output format (defaults to `table`).

## Subcommands

| Command | Description |
| --- | --- |
| `repo list` | List repositories in a workspace |
| `repo view` | View repository details |
| `repo clone` | Clone a repository |
| `repo create` | Create a new repository |
| `repo update` | Update repository settings |
| `repo move` | Move a repository to a different project in its workspace |
| `repo fork` | Fork a repository |
| `repo delete` | Delete a repository |
| `repo watchers` | List users watching a repository |
| `repo forks` | List forks of a repository |
| `repo download` | Manage repository downloads (uploaded file artifacts) |
| `repo branch` | Manage repository branches |
| `repo tag` | Manage repository tags |
| `repo commit` | Work with repository commits |
| `repo file` | Browse repository files and file contents |
| `repo webhook` | Manage repository webhooks |
| `repo branch-restriction` | Manage branch restrictions (branch permissions) |
| `repo reviewer` | Manage default reviewers |
| `repo permission` | Inspect repository user and group permissions |
| `repo deploy-key` | Manage repository deploy keys |
| `repo environment` | Manage deployment environments |
| `repo deployment` | View repository deployments |
| `repo branching-model` | Manage the repository branching model |

### `repo download`

Manage repository downloads (uploaded file artifacts).

| Command | Description |
| --- | --- |
| `repo download upload` | Upload one or more files to the repository's downloads area |
| `repo download list` | List artifacts in the repository's downloads area |
| `repo download get` | Download an artifact from the repository's downloads area |
| `repo download delete` | Delete an artifact from the repository's downloads area |

### `repo branch`

Manage repository branches.

| Command | Description |
| --- | --- |
| `repo branch list` | List branches in a repository |
| `repo branch create` | Create a branch |
| `repo branch view` | View a single branch |
| `repo branch delete` | Delete a branch |

### `repo tag`

Manage repository tags.

| Command | Description |
| --- | --- |
| `repo tag list` | List tags in a repository |
| `repo tag create` | Create a tag |
| `repo tag view` | View a single tag |
| `repo tag delete` | Delete a tag |

### `repo commit`

Work with repository commits.

| Command | Description |
| --- | --- |
| `repo commit list` | List commits in a repository |
| `repo commit view` | View a single commit |
| `repo commit diff` | Show the raw diff for a commit or revision spec |

### `repo file`

Browse repository files and file contents.

| Command | Description |
| --- | --- |
| `repo file ls` | List files and directories in a repository |
| `repo file cat` | Print the raw contents of a file |
| `repo file history` | List the commits that touched a file |

### `repo webhook`

Manage repository webhooks.

| Command | Description |
| --- | --- |
| `repo webhook list` | List webhooks on a repository |
| `repo webhook create` | Create a webhook |
| `repo webhook view` | View webhook details |
| `repo webhook update` | Update a webhook (unset fields keep their current values) |
| `repo webhook delete` | Delete a webhook |

### `repo branch-restriction`

Manage branch restrictions (branch permissions).

| Command | Description |
| --- | --- |
| `repo branch-restriction list` | List branch restriction rules on a repository |
| `repo branch-restriction create` | Create a branch restriction rule |
| `repo branch-restriction delete` | Delete a branch restriction rule |

### `repo reviewer`

Manage default reviewers.

| Command | Description |
| --- | --- |
| `repo reviewer list` | List a repository's default reviewers |
| `repo reviewer add` | Add a user to the default reviewers |
| `repo reviewer remove` | Remove a user from the default reviewers |

### `repo permission`

Inspect repository user and group permissions.

| Command | Description |
| --- | --- |
| `repo permission list-users` | List users with explicit permissions on a repository |
| `repo permission list-groups` | List groups with explicit permissions on a repository |
| `repo permission grant-user` | Grant or update a user's permission |
| `repo permission revoke-user` | Revoke a user's explicit permission |
| `repo permission grant-group` | Grant or update a group's permission |
| `repo permission revoke-group` | Revoke a group's explicit permission |

### `repo deploy-key`

Manage repository deploy keys.

| Command | Description |
| --- | --- |
| `repo deploy-key list` | List deploy keys for a repository |
| `repo deploy-key add` | Add a deploy key to a repository |
| `repo deploy-key delete` | Delete a deploy key from a repository |

### `repo environment`

Manage deployment environments.

| Command | Description |
| --- | --- |
| `repo environment list` | List deployment environments for a repository |
| `repo environment create` | Create a deployment environment |
| `repo environment delete` | Delete a deployment environment |

### `repo deployment`

View repository deployments.

| Command | Description |
| --- | --- |
| `repo deployment list` | List deployments for a repository |
| `repo deployment view` | View deployment details |

### `repo branching-model`

Manage the repository branching model.

| Command | Description |
| --- | --- |
| `repo branching-model view` | View the repository's effective branching model |
| `repo branching-model set` | Update the repository's branching model settings |

## Examples

List repositories in a workspace, filtered with a Bitbucket query (BBQL) expression and sorted:

```bash
bitbucket repo list myworkspace --query 'language="rust"' --sort -updated_on
```

View a repository's details:

```bash
bitbucket repo view myworkspace/myrepo
```

Create a new private repository in a project:

```bash
bitbucket repo create myworkspace/myrepo --description "My new repo" --project PROJ --language rust
```

Update repository settings:

```bash
bitbucket repo update myworkspace/myrepo --private --description "Updated description"
```

Create a branch from `main`:

```bash
bitbucket repo branch create myworkspace/myrepo feature/new-thing --from main
```

Create an annotated tag pointing at a branch tip:

```bash
bitbucket repo tag create myworkspace/myrepo v1.0.0 --from main --message "First release"
```

Create a webhook subscribed to multiple events:

```bash
bitbucket repo webhook create myworkspace/myrepo \
  --url https://ci.example.com/hook \
  --event repo:push \
  --event pullrequest:created
```

Upload a build artifact to the repository's downloads area:

```bash
bitbucket repo download upload myworkspace/myrepo ./dist/app-linux.tar.gz
```

Grant a user write permission on a repository:

```bash
bitbucket repo permission grant-user myworkspace/myrepo 557058:1234abcd-... --permission write
```
