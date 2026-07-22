---
title: Configuration
description: Configure default workspace and global options for Bitbucket CLI.
---

Configuration is stored at `~/.config/bitbucket-cli/config.toml`:

```toml
[auth]
username = "your-username"
default_workspace = "your-workspace"
```

## Keys

| Key | Description |
| --- | --- |
| `auth.username` | The authenticated account's username, set during login. |
| `auth.default_workspace` | Workspace used when a command omits one. This is the primary default-workspace key. |
| `defaults.workspace` | Legacy fallback for the default workspace, read only when `auth.default_workspace` is unset. Prefer `auth.default_workspace`. |

## Resolving the workspace

Commands that take a `workspace/repo-slug` argument accept a bare repository slug and resolve the workspace in this order:

1. The `--workspace` global flag.
2. `auth.default_workspace` from the config file.
3. `defaults.workspace` (legacy fallback).

```bash
# With a default workspace configured, both of these work:
bitbucket repo view myworkspace/myrepo
bitbucket repo view myrepo
```

## Global flags

These flags are available on every command:

| Flag | Description |
| --- | --- |
| `--workspace <name>` | Workspace to use when omitted from arguments. |
| `--output <table\|json>` | Output format. `json` emits machine-readable output — see [Scripting with JSON](/bitbucket-cli/scripting/). |

## Environment variables

| Variable | Purpose |
| --- | --- |
| `BITBUCKET_CLIENT_ID` / `BITBUCKET_CLIENT_SECRET` | OAuth consumer credentials for `auth login --oauth`. |
| `BITBUCKET_EMAIL` / `BITBUCKET_API_TOKEN` | Account email and API token for `auth login --api-key`. |
