# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
  `models::Workspace` gained the `is_private` and `created_on` fields.
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
