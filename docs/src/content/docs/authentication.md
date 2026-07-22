---
title: Authentication
description: Sign in to Bitbucket Cloud with OAuth 2.0 or an API token.
---

Bitbucket CLI supports two authentication methods. **OAuth 2.0 is recommended** — it is more secure and refreshes tokens automatically. API tokens are the fallback for CI and automation.

:::note
Atlassian has deprecated app passwords. Bitbucket CLI uses OAuth 2.0 and API tokens (HTTP access tokens) only.
:::

## Option A — OAuth 2.0 (recommended)

```bash
bitbucket auth login --oauth
```

You'll need an OAuth consumer first:

1. Go to your [Bitbucket workspace settings](https://bitbucket.org/[workspace]/workspace/settings/oauth-consumers/new).
2. Set the callback URL to **one** of the following — the CLI uses the first available port:
   - `http://127.0.0.1:8080/callback`
   - `http://127.0.0.1:3000/callback`
   - `http://127.0.0.1:8888/callback`
   - `http://127.0.0.1:9000/callback`
3. Grant these permissions: Account (Read), Repositories (Read), Pull requests (Read/Write), Issues (Read/Write), Pipelines (Read/Write).
4. Copy the **Key** (client ID) and **Secret** when prompted.

You can also pass the consumer credentials non-interactively with `--client-id` / `--client-secret`, or via the `BITBUCKET_CLIENT_ID` and `BITBUCKET_CLIENT_SECRET` environment variables.

## Option B — API token (CI / automation)

```bash
bitbucket auth login --api-key
```

Create the token first:

1. Open your [Bitbucket personal settings](https://bitbucket.org/account/settings/) and go to the API tokens section.
2. Create a token with the permissions you need.
3. Enter your Atlassian account email and the token when prompted, or run non-interactively:

```bash
bitbucket auth login --api-key --email you@example.com --token <api-token>
```

The email and token can also be supplied via `BITBUCKET_EMAIL` and `BITBUCKET_API_TOKEN`.

## Checking and clearing credentials

```bash
bitbucket auth status   # show the authenticated account
bitbucket auth logout   # remove stored credentials
```

## Credential storage

Credentials are stored in your operating system's keyring when one is available. On systems without a keyring (for example, headless CI), they fall back to a restricted-permission file at `~/.config/bitbucket/credentials.json` (mode `0600` on Unix).
