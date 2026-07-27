# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.1] - 2026-07-27

### Changed

- Repository transferred to `github.com/quinnjr/bitbucket-cli`. The crate's
  `repository`, `homepage`, and `documentation` metadata, along with all badges,
  install commands, and packaging sources, now point at the new location; the
  documentation site moved to <https://quinnjr.github.io/bitbucket-cli/>.
- Rebranded from Pegasus Heavy Industries to `quinnjr`; copyright is now held by
  Joseph Quinn, and support/funding links point to <https://quinnjr.dev>.
- Rebuilt the documentation site with Astro + Starlight and adopted the Git Flow
  branching model.

## [1.0.0] - 2026-07-21

### Added

- Global `--output <table|json>` flag on every command; `--output json` emits
  raw pretty-printed JSON for scripting.
- New top-level `user` command: `whoami`, `view`, `emails`.
- repo: new subcommand groups `branch`, `tag`, `commit`, `file`, `webhook`,
  `branch-restriction`, `reviewer`, `permission`, `deploy-key`, `environment`,
  `deployment`, and `branching-model`; new `watchers` and `forks` commands;
  `download get`; `list` filters (`--role`, `-q/--query`, `--sort`, `--page`);
  `create` flags `--language`, `--issues`, `--wiki`, `--website`, and
  `--main-branch`; `update --website`.
- pr: new `edit`, `unapprove`, `request-changes`, `unrequest-changes`,
  `commits`, `statuses`, `diffstat`, `activity`, and `patch` commands; a `task`
  group (`list`, `add`, `resolve`, `reopen`, `delete`); comment management
  (`edit-comment`, `delete-comment`, `resolve-comment`, `unresolve-comment`);
  `create --reviewers`; `list` accepts a repeatable `--state` plus
  `-q/--query`, `--sort`, and `--page`; `comment --path/--line/--parent` for
  inline and threaded comments.
- issue: new `edit`, `delete`, `comments`, `vote`/`unvote`, `watch`/`unwatch`,
  `changes`, `components`, `milestones`, `versions`, an `attachment` group
  (`list`, `add`, `delete`), `edit-comment`, and `delete-comment` commands;
  `list` filters (`--kind`, `--priority`, `--assignee`, `--reporter`,
  `-q/--query`, `--sort`, `--page`);
  `create --assignee/--component/--milestone/--version`; `view --comments`.
- pipeline: `trigger --var/--secured-var/--commit`; `list --status/
  --target-branch/--sort/--page`; `view --step/--full-logs`; `stop --uuid`;
  new `variable`, `workspace-variable`, `schedule`, `cache`, `config`, and
  `rerun` commands.
- workspace: new `view`, `members`, and `permissions` commands and a `project`
  group (`list`, `view`, `create`, `edit`, `delete`); `list` gained `--role`,
  `-q/--query`, `--sort`, `--limit`, and `--all`.

### Changed

- `workspace list` now fetches a single page by default and supports filter
  flags; pass `--all` to fetch every page as before.
- `issue list` now composes the individual filter flags into a single BBQL
  `q=` expression; `--query` overrides the individual flags.
- Library API: `BitbucketClient::update_pull_request` now takes an
  `UpdatePullRequestRequest` body; `BitbucketClient::list_issue_comments` now
  takes a `pagelen: Option<u32>` argument; `models::UserRef` has been reshaped
  to optional `uuid`/`username` fields. Public request/filter structs are now
  `#[non_exhaustive]`, and the internal `*_filtered` client methods and their
  filter structs are `pub(crate)`, so the stable library surface is smaller.
- Config: the unused `[defaults] repository` and `[display]` (`color`, `pager`)
  keys have been removed; they were never read.

### Fixed

- `pr checkout` no longer fetches from the wrong repository: it verifies that
  `origin` points at the PR's repository, refuses PRs that come from forks
  (with instructions to fetch manually), and fast-forwards a stale local
  branch to the fetched tip instead of silently reusing it.
- `pipeline trigger --wait` no longer hangs forever on pipelines that enter
  the Paused state; it now reports that a manual resume is required.
- The `User-Agent` header now carries the real crate version.
- `UserLinks.self` now deserializes correctly from the API's `self` key.

### Removed

- Dead `get_main_branch` and `list_branches` functions from the repos API
  module (superseded by the refs API).
- Dead `defaults.repository` config field.
- Dead TUI `load_all_data` method.

## [0.4.0] - 2026-07-21

### Breaking

- The `-w` short flag has been removed from the global `--workspace` option; use
  `--workspace` instead. Local `-w` shorts on subcommands (e.g. `repo view --web`,
  `repo fork --workspace`) are unchanged.
- The global `-r`/`--repo` flag has been removed. It was never functional; pass a
  `workspace/repo-slug` argument or use the new bare-slug fallback instead.
- Repository arguments with empty path components (e.g. `workspace/` or `/repo`)
  are now rejected instead of being passed through.
- Library API: the `Cli::repo` field has been removed; the
  `tui::views::{dashboard,repos,prs,issues}` modules have been removed;
  `models::Workspace` gained the `is_private` and `created_on` fields;
  `BitbucketClient::list_pr_comments` has been replaced by
  `list_recent_pr_comments` (bounded, newest-first).
- The Arch release asset has been renamed from
  `bitbucket-cli-<version>-x86_64.pkg.tar.zst` to
  `bitbucket-cli-<version>-x86_64-linux.tar.zst`. It is a plain binary tarball,
  not a pacman package; use `packaging/arch/PKGBUILD` to build a real package.
- `issue list --state` filtering is now sent as Bitbucket's `q=` BBQL query
  parameter instead of a bare `state=` parameter.

### Added

- `workspace list` command.
- `repo update` command.
- `repo move` command.
- Bare repo-slug arguments (e.g. `repo view my-repo`) now resolve against
  `--workspace` or the configured default workspace.
- `repo list` accepts an omitted workspace, and `repo create` accepts
  `workspace/name` or a bare name, resolving the workspace the same way.
- `repo download` — upload, list, and delete repository download artifacts
  (contributed in #69).

### Fixed

- Debug-build panic on `repo view`, `pr view`, and `issue view` caused by the
  `-w` short-flag collision.
- The default workspace is now read from `[auth] default_workspace` (the key
  documented in the README), with `[defaults] workspace` kept as a legacy
  fallback.
- The OAuth callback now requires the CSRF `state` parameter and fails cleanly
  when authorization is denied instead of hanging.
- `pipeline view --build` and `pipeline stop --build` now find builds beyond the
  100 most recent.
- The TUI loads repositories once and surfaces load failures instead of silently
  showing an empty list.
- Packaging versions realigned across Cargo.toml, the Windows MSI config, and
  the Arch/Alpine package files.
- Docs corrected: supported auth methods, config file path, versions, and links.
