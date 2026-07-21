# Persist OAuth Consumer Credentials

## Problem

Every `bitbucket auth login --oauth` prompts for client_id and client_secret,
even though these are stable, long-lived app credentials that don't change
between sessions. Users must re-enter them (or set env vars) every time they
need to re-authenticate, which typically happens when the access token expires
and there's no refresh token.

## Solution

Store `client_id` and `client_secret` in the existing `credentials.json`
alongside access/refresh tokens. On subsequent logins, read them from storage
and skip the interactive prompts — go straight to opening the browser.

## Priority Chain for Consumer Credentials

1. CLI flags (`--client-id`, `--client-secret`)
2. Env vars (`BITBUCKET_CLIENT_ID`, `BITBUCKET_CLIENT_SECRET`)
3. Stored credentials from previous login
4. Interactive prompt (first-time only)

## Changes

### 1. `src/auth/mod.rs` — Credential::OAuth variant

Add optional `client_id` and `client_secret` fields with `#[serde(default)]`
for backward compatibility with existing credentials.json files.

### 2. `src/cli/auth.rs` — Login command

Before prompting interactively, check if stored credentials contain consumer
creds and use those as a fallback in the priority chain.

### 3. `src/auth/oauth.rs` — OAuthFlow::authenticate

Include consumer creds when constructing the `Credential::OAuth` that gets
stored after successful authentication.

## Backward Compatibility

Existing credentials.json files without `client_id`/`client_secret` will
deserialize with `None` values via `#[serde(default)]`. The login flow will
fall through to the interactive prompt as before.
