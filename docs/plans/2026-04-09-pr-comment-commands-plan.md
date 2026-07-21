# PR Comment Commands Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `list-comments` and `view-comment` subcommands to `bitbucket pr` for browsing and inspecting pull request comments.

**Architecture:** Two new variants in `PrCommands` enum, one new API method (`get_pr_comment`), no model changes. Follows the existing command → API → model pattern exactly.

**Tech Stack:** Rust, clap (CLI), reqwest (HTTP), tabled (tables), colored (terminal colors), chrono (dates), serde (serialization)

---

## File Map

| File | Action | Responsibility |
|------|--------|---------------|
| `src/api/pullrequests.rs` | Modify | Add `get_pr_comment()` method |
| `src/cli/pr.rs` | Modify | Add `ListComments` and `ViewComment` variants, `CommentRow` table struct, handlers |

No new files. No model changes — `PullRequestComment`, `CommentContent`, `InlineComment`, `CommentLinks` already exist in `src/models/pr.rs:174-211`.

---

### Task 1: Add `get_pr_comment()` API method

**Files:**
- Modify: `src/api/pullrequests.rs:151-163` (after existing `list_pr_comments`)

- [ ] **Step 1: Add the method**

Add this method to `impl BitbucketClient` in `src/api/pullrequests.rs`, directly after the `list_pr_comments` method (after line 163):

```rust
    /// Get a specific comment on a pull request
    pub async fn get_pr_comment(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
        comment_id: u64,
    ) -> Result<PullRequestComment> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/comments/{}",
            workspace, repo_slug, pr_id, comment_id
        );
        self.get(&path).await
    }
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check`
Expected: compiles with no errors

- [ ] **Step 3: Commit**

```bash
git add src/api/pullrequests.rs
git commit -m "feat(api): add get_pr_comment endpoint

The existing PR comment API only supports listing all comments and
adding new ones. There is no way to fetch a single comment by ID,
which the upcoming view-comment CLI command needs.

Add get_pr_comment() which hits GET /repositories/{workspace}/{repo}/
pullrequests/{pr_id}/comments/{comment_id} and returns a single
PullRequestComment."
```

---

### Task 2: Add `ListComments` command variant and `CommentRow` table struct

**Files:**
- Modify: `src/cli/pr.rs:12-136` (enum variants), `src/cli/pr.rs:174-186` (after PrRow struct)

- [ ] **Step 1: Add `ListComments` variant to `PrCommands` enum**

In `src/cli/pr.rs`, add this variant after the existing `Comment` variant (after line 135, before the closing `}`):

```rust
    /// List comments on a pull request
    ListComments {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        id: u64,

        /// Number of results
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },
```

- [ ] **Step 2: Add `CommentRow` table struct**

In `src/cli/pr.rs`, add this struct after the `PrRow` struct (after line 186):

```rust
#[derive(Tabled)]
struct CommentRow {
    #[tabled(rename = "ID")]
    id: u64,
    #[tabled(rename = "AUTHOR")]
    author: String,
    #[tabled(rename = "CREATED")]
    created: String,
    #[tabled(rename = "TYPE")]
    comment_type: String,
    #[tabled(rename = "CONTENT")]
    content: String,
}
```

- [ ] **Step 3: Add handler for `ListComments`**

In the `match self` block inside `PrCommands::run()`, add this arm after the `PrCommands::Comment` handler (after line 465):

```rust
            PrCommands::ListComments { repo, id, limit } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let comments = client
                    .list_pr_comments(&workspace, &repo_slug, id)
                    .await?;

                let mut values: Vec<_> = comments.values.into_iter().take(limit as usize).collect();

                if values.is_empty() {
                    println!("No comments found");
                    return Ok(());
                }

                values.sort_by_key(|c| c.created_on);

                let rows: Vec<CommentRow> = values
                    .iter()
                    .map(|c| CommentRow {
                        id: c.id,
                        author: c.user.display_name.clone(),
                        created: c.created_on.format("%Y-%m-%d %H:%M").to_string(),
                        comment_type: if c.inline.is_some() {
                            "inline".to_string()
                        } else {
                            "general".to_string()
                        },
                        content: c.content.raw.chars().take(50).collect(),
                    })
                    .collect();

                let table = Table::new(rows).to_string();
                println!("{}", table);

                Ok(())
            }
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo check`
Expected: compiles with no errors

- [ ] **Step 5: Commit**

```bash
git add src/cli/pr.rs
git commit -m "feat(cli): add list-comments subcommand for pull requests

There was no way to see existing comments on a PR from the CLI.
The only comment-related command was 'pr comment' which adds a
new comment.

Add 'bitbucket pr list-comments <repo> <pr_id>' which displays a
table of all comments with ID, author, date, type (inline vs
general), and a truncated content preview. Results are sorted by
creation date and capped by --limit (default 25)."
```

---

### Task 3: Add `ViewComment` command variant and handler

**Files:**
- Modify: `src/cli/pr.rs` (enum variants, match block)

- [ ] **Step 1: Add `ViewComment` variant to `PrCommands` enum**

In `src/cli/pr.rs`, add this variant after the `ListComments` variant just added:

```rust
    /// View a specific comment on a pull request
    ViewComment {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        #[arg(value_name = "PR_ID")]
        id: u64,

        /// Comment ID
        comment_id: u64,
    },
```

- [ ] **Step 2: Add handler for `ViewComment`**

In the `match self` block inside `PrCommands::run()`, add this arm after the `ListComments` handler:

```rust
            PrCommands::ViewComment {
                repo,
                id,
                comment_id,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let comment = client
                    .get_pr_comment(&workspace, &repo_slug, id, comment_id)
                    .await?;

                println!(
                    "{} #{} on PR #{}",
                    "Comment".bold(),
                    comment.id,
                    id
                );
                println!("{}", "─".repeat(60));

                println!("{} {}", "Author:".dimmed(), comment.user.display_name);
                println!(
                    "{} {}",
                    "Created:".dimmed(),
                    comment.created_on.format("%Y-%m-%d %H:%M")
                );

                if let Some(updated) = comment.updated_on {
                    println!(
                        "{} {}",
                        "Updated:".dimmed(),
                        updated.format("%Y-%m-%d %H:%M")
                    );
                }

                if let Some(inline) = &comment.inline {
                    let line = inline.to.or(inline.from);
                    let location = match line {
                        Some(l) => format!("{}:{}", inline.path, l),
                        None => inline.path.clone(),
                    };
                    println!("{} {}", "Type:".dimmed(), "inline");
                    println!("{} {}", "File:".dimmed(), location.cyan());
                } else {
                    println!("{} {}", "Type:".dimmed(), "general");
                }

                println!();
                println!("{}", comment.content.raw);

                if let Some(links) = &comment.links {
                    if let Some(html) = &links.html {
                        println!();
                        println!("{} {}", "URL:".dimmed(), html.href.cyan());
                    }
                }

                Ok(())
            }
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check`
Expected: compiles with no errors

- [ ] **Step 4: Commit**

```bash
git add src/cli/pr.rs
git commit -m "feat(cli): add view-comment subcommand for pull requests

The list-comments command shows truncated previews. When you have
a specific comment ID (from a notification, API response, or the
list-comments output) there was no way to see the full content and
metadata.

Add 'bitbucket pr view-comment <repo> <pr_id> <comment_id>' which
displays the full comment with author, timestamps, type, inline
file location (for code comments), full untruncated body, and a
link to the comment in the browser."
```

---

### Task 4: Build and verify

- [ ] **Step 1: Full build**

Run: `cargo build`
Expected: compiles with no errors and no warnings

- [ ] **Step 2: Verify help text**

Run: `cargo run -- pr --help`
Expected: `list-comments` and `view-comment` appear in the subcommand list

Run: `cargo run -- pr list-comments --help`
Expected: shows repo, id positional args and --limit option

Run: `cargo run -- pr view-comment --help`
Expected: shows repo, id, comment_id positional args
