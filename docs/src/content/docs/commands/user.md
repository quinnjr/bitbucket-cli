---
title: user
description: Inspect Bitbucket user accounts.
sidebar:
  order: 6
---

The `bitbucket user` command group inspects Bitbucket user accounts — show who you are authenticated as, look up another user's public profile, and list the email addresses on your account.

## Subcommands

| Command | Description |
| --- | --- |
| `user whoami` | Show the currently authenticated user |
| `user view` | View a user's public profile |
| `user emails` | List email addresses on the authenticated account |

`user view` takes an `<account>` argument — the account ID or UUID (including braces) of the user. All three commands accept the global `--workspace` and `--output` flags.

## Examples

Show the currently authenticated user:

```bash
bitbucket user whoami
```

Look up another user by account ID:

```bash
bitbucket user view 557058:1a2b3c4d-5e6f-7a8b-9c0d-1e2f3a4b5c6d
```

Look up a user by UUID (braces included):

```bash
bitbucket user view '{1a2b3c4d-5e6f-7a8b-9c0d-1e2f3a4b5c6d}'
```

List the email addresses on your account:

```bash
bitbucket user emails
```

Get your account UUID for scripting:

```bash
bitbucket user whoami --output json | jq -r '.uuid'
```
