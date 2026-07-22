---
title: issue
description: Create, edit, and track issues, comments, and attachments.
sidebar:
  order: 3
---

The `bitbucket issue` command group manages a repository's issue tracker: filing and editing issues, threading comments, following change logs, voting and watching, and attaching files. Every subcommand takes a repository as `workspace/repo-slug` (a bare slug resolves the workspace from `--workspace` or your config default) and honors the global `--output table|json` flag.

## Subcommands

| Command | Description |
| --- | --- |
| `issue list` | List issues |
| `issue view` | View issue details |
| `issue create` | Create a new issue |
| `issue edit` | Edit an issue's fields |
| `issue delete` | Delete an issue |
| `issue comment` | Add a comment to an issue |
| `issue comments` | List comments on an issue |
| `issue edit-comment` | Edit a comment on an issue |
| `issue delete-comment` | Delete a comment from an issue |
| `issue changes` | List the change log of an issue |
| `issue vote` | Vote for an issue |
| `issue unvote` | Remove your vote from an issue |
| `issue watch` | Watch an issue |
| `issue unwatch` | Stop watching an issue |
| `issue components` | List the components defined in the issue tracker |
| `issue milestones` | List the milestones defined in the issue tracker |
| `issue versions` | List the versions defined in the issue tracker |
| `issue attachment` | Manage files attached to an issue |
| `issue close` | Close an issue |
| `issue reopen` | Reopen an issue |

### Common flags

`list` accepts filters `-s, --state` (`new`, `open`, `resolved`, `on-hold`, `invalid`, `duplicate`, `wontfix`, `closed`), `-k, --kind` (`bug`, `enhancement`, `proposal`, `task`), `-p, --priority` (`trivial`, `minor`, `major`, `critical`, `blocker`), `--assignee`, and `--reporter` (both account IDs), plus `-q, --query` for a raw BBQL query that overrides the individual filters, `--sort` (e.g. `-updated_on`), `--page`, and `-l, --limit` (default `25`, capped at 100).

`create` requires `-t, --title` and defaults `-k, --kind` to `bug` and `-p, --priority` to `major`. It also takes `-b, --body`, `-a, --assignee`, `-c, --component`, `-m, --milestone`, and `--version`. `edit` mirrors these fields plus `-s, --state`, changing only the flags you pass.

`view` adds `-c, --comments` to include the comment thread and `-w, --web` to open the issue in a browser. `comments` and `changes` support `-l, --limit`. Destructive commands (`delete`, `delete-comment`, `attachment delete`) prompt for confirmation unless you pass `-y, --yes`.

### `issue attachment`

Manage files attached to an issue.

| Command | Description |
| --- | --- |
| `issue attachment list` | List files attached to an issue |
| `issue attachment add` | Attach one or more files to an issue |
| `issue attachment delete` | Delete an attachment from an issue |

`attachment add` takes one or more file paths. `attachment delete` takes the attachment's `<PATH>` (the filename shown by `issue attachment list`) and accepts `-y, --yes` to skip the confirmation prompt.

## Examples

List open bugs assigned to a specific account:

```bash
bitbucket issue list myworkspace/myrepo --kind bug --assignee 557058:1a2b3c4d
```

Filter with a raw BBQL query instead of the individual flags:

```bash
bitbucket issue list myworkspace/myrepo -q 'state="open" AND priority="critical"'
```

Create a critical task and assign it:

```bash
bitbucket issue create myworkspace/myrepo \
  --title "Fix flaky auth test" \
  --assignee 557058:1a2b3c4d \
  --priority critical
```

Mark an issue as resolved:

```bash
bitbucket issue edit myworkspace/myrepo 42 --state resolved
```

Comment on an issue:

```bash
bitbucket issue comment myworkspace/myrepo 42 --body "Reproduced on main, patch incoming."
```

Attach log files to an issue:

```bash
bitbucket issue attachment add myworkspace/myrepo 42 ./error.log ./stacktrace.txt
```

Review an issue's change log:

```bash
bitbucket issue changes myworkspace/myrepo 42
```
