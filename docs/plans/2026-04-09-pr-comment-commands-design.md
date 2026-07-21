# PR Comment Commands Design

## Overview

Add two new subcommands under `bitbucket pr` for viewing pull request comments:

- `list-comments` — tabular listing of all comments on a PR
- `view-comment` — detailed view of a single comment by ID

## Commands

### `bitbucket pr list-comments <repo> <pr_id>`

Lists all comments on a pull request in a table.

**Arguments:**
- `repo` (positional, required): `workspace/repo-slug`
- `pr_id` (positional, required): pull request ID (`u64`)
- `--limit` / `-l` (optional, default 25): max results

**Output columns:**

| ID | AUTHOR | CREATED | TYPE | CONTENT |
|----|--------|---------|------|---------|
| 42 | Alice  | 2026-04-01 | inline | Fix the off-by-one error in this loop... |
| 43 | Bob    | 2026-04-02 | general | Looks good overall, just one nit... |

- `CONTENT` truncated to 50 characters
- `TYPE` is "inline" if `inline` field is present, "general" otherwise
- Empty list prints "No comments found"

### `bitbucket pr view-comment <repo> <pr_id> <comment_id>`

Displays full details of a single comment.

**Arguments:**
- `repo` (positional, required): `workspace/repo-slug`
- `pr_id` (positional, required): pull request ID (`u64`)
- `comment_id` (positional, required): comment ID (`u64`)

**Output format** (matches `pr view` style):

```
Comment #42 on PR #7
──────────────────────────────────────────────────────────────
Author:  Alice
Created: 2026-04-01 14:30
Updated: 2026-04-01 15:00
Type:    inline
File:    src/main.rs:42

Fix the off-by-one error in this loop body. The index
should start at 0, not 1.

URL: https://bitbucket.org/workspace/repo/pull-requests/7/_/diff#comment-42
```

- `Updated` line omitted if `updated_on` is `None`
- `File` line only shown for inline comments, format `path:line` using `inline.path` and `inline.to` (or `inline.from` if `to` is `None`)
- `URL` from `links.html.href` if present

## API Layer

### New method: `get_pr_comment()`

```
GET /repositories/{workspace}/{repo_slug}/pullrequests/{pr_id}/comments/{comment_id}
```

Returns `PullRequestComment`. Added to `src/api/pullrequests.rs`.

The existing `list_pr_comments()` method is already sufficient for `list-comments`.

## Models

No model changes needed. `PullRequestComment`, `CommentContent`, `InlineComment`, `CommentRef`, and `CommentLinks` already exist in `src/models/pr.rs`.

## Files Modified

1. `src/api/pullrequests.rs` — add `get_pr_comment()` method
2. `src/cli/pr.rs` — add `ListComments` and `ViewComment` variants to `PrCommands`, add handlers, add `CommentRow` table struct
