---
title: pr
description: Create, review, merge, and manage pull requests.
sidebar:
  order: 2
---

The `bitbucket pr` command group covers the full pull request lifecycle — opening and editing PRs, reviewing them with approvals and inline comments, tracking build statuses and pipelines, managing review tasks, and merging or declining. Every subcommand takes the repository as `workspace/repo-slug` (the workspace can be omitted when a default is configured with `--workspace` or in your config).

## Subcommands

| Command | Description |
| --- | --- |
| `pr list` | List pull requests |
| `pr view` | View pull request details |
| `pr create` | Create a new pull request |
| `pr edit` | Edit an existing pull request |
| `pr merge` | Merge a pull request |
| `pr approve` | Approve a pull request |
| `pr unapprove` | Remove your approval from a pull request |
| `pr request-changes` | Request changes on a pull request |
| `pr unrequest-changes` | Withdraw your request for changes on a pull request |
| `pr decline` | Decline a pull request |
| `pr checkout` | Checkout a pull request branch locally |
| `pr diff` | View pull request diff |
| `pr comment` | Add a comment to a pull request |
| `pr edit-comment` | Edit a comment on a pull request |
| `pr delete-comment` | Delete a comment on a pull request |
| `pr resolve-comment` | Resolve a comment thread on a pull request |
| `pr unresolve-comment` | Reopen a resolved comment thread on a pull request |
| `pr list-comments` | List comments on a pull request |
| `pr view-comment` | View a specific comment on a pull request |
| `pr pipelines` | List pipelines for the PR's head commit |
| `pr commits` | List commits on a pull request |
| `pr statuses` | Show build statuses for a pull request |
| `pr diffstat` | Show the per-file change summary for a pull request |
| `pr task` | Manage tasks on a pull request |
| `pr activity` | Show the activity feed for a pull request |
| `pr patch` | Print the patch (mbox-style) for a pull request |

### `pr task`

Manage review tasks (checklist items) on a pull request.

| Command | Description |
| --- | --- |
| `pr task list` | List tasks on a pull request |
| `pr task add` | Add a task to a pull request |
| `pr task resolve` | Mark a task as resolved |
| `pr task reopen` | Reopen a resolved task |
| `pr task delete` | Delete a task from a pull request |

## Examples

List pull requests, filtering by state. The `--state` flag is repeatable to match several states at once:

```bash
bitbucket pr list myworkspace/myrepo --state open --state merged
```

Create a pull request from a feature branch, adding reviewers:

```bash
bitbucket pr create myworkspace/myrepo \
  --title "Add rate limiting to the API" \
  --source feature/rate-limit \
  --reviewers alice,bob
```

Edit an existing pull request's title and description:

```bash
bitbucket pr edit myworkspace/myrepo 42 \
  --title "Add configurable rate limiting" \
  --body "Adds per-client limits with a token bucket."
```

Merge a pull request with the squash strategy:

```bash
bitbucket pr merge myworkspace/myrepo 42 --strategy squash --close-source-branch
```

Add an inline comment anchored to a specific file and line:

```bash
bitbucket pr comment myworkspace/myrepo 42 \
  --body "This should handle the empty-slice case." \
  --path src/limiter.rs \
  --line 87
```

Add a review task to a pull request:

```bash
bitbucket pr task add myworkspace/myrepo 42 --body "Add a regression test for the retry path"
```

Show the per-file change summary for a pull request:

```bash
bitbucket pr diffstat myworkspace/myrepo 42
```

Approve a pull request:

```bash
bitbucket pr approve myworkspace/myrepo 42
```
