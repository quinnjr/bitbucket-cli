---
title: auth
description: Authenticate with Bitbucket using OAuth 2.0 or an API token.
sidebar:
  order: 7
---

The `bitbucket auth` command group manages authentication with Bitbucket — sign in with OAuth 2.0 or an API key, check your current authentication status, and remove stored credentials.

:::tip
For the full walkthrough (creating an OAuth consumer, credential storage), see the [Authentication guide](/bitbucket-cli/authentication/).
:::

## Subcommands

| Command | Description |
| --- | --- |
| `auth login` | Authenticate with Bitbucket (OAuth 2.0 or API key) |
| `auth logout` | Remove stored credentials |
| `auth status` | Show authentication status |

## Login options

Flags for `bitbucket auth login`. Credentials can also be supplied through the environment variables shown below.

| Flag | Environment variable | Description |
| --- | --- | --- |
| `--oauth` | — | Use OAuth 2.0 authentication (interactive browser sign-in) |
| `--api-key` | — | Use API key authentication (HTTP access token; for automation/CI) |
| `--email <EMAIL>` | `BITBUCKET_EMAIL` | Atlassian account email / Bitbucket username (for API key authentication) |
| `--token <TOKEN>` | `BITBUCKET_API_TOKEN` | API key (HTTP access token; for API key authentication, implies `--api-key`) |
| `--client-id <CLIENT_ID>` | `BITBUCKET_CLIENT_ID` | OAuth Client ID (for OAuth authentication) |
| `--client-secret <CLIENT_SECRET>` | `BITBUCKET_CLIENT_SECRET` | OAuth Client Secret (for OAuth authentication) |

## Examples

Sign in interactively with OAuth 2.0 (opens your browser):

```bash
bitbucket auth login --oauth
```

Sign in non-interactively with an API key — ideal for CI:

```bash
bitbucket auth login --email you@example.com --token "$BITBUCKET_API_TOKEN"
```

Check your current authentication status:

```bash
bitbucket auth status
```

Remove stored credentials:

```bash
bitbucket auth logout
```
