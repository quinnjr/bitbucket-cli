---
title: Scripting with JSON
description: Use --output json for machine-readable output you can pipe into jq and automation.
---

Every command accepts a global `--output` flag. The default is `table` (human-readable); `json` emits raw, pretty-printed JSON suitable for scripting.

```bash
bitbucket repo list myworkspace --output json
```

## How JSON mode behaves

- **List commands** print the array of items (an empty result is `[]`), with no "more available" notices.
- **View commands** print the single entity.
- **Mutations** print the returned entity, or `{"ok": true}` when the API returns no body.
- **Errors** go to stderr as text, so stdout always contains valid JSON (or nothing).
- **Confirmation prompts** still run for destructive commands; pass `--yes` for non-interactive use.
- **Raw-output commands** — `pr diff`, `pr patch`, `commit diff`, `file cat` — emit their raw text unchanged so they stay pipe-friendly.

## Examples

Get the clone URL of every repository in a workspace:

```bash
bitbucket repo list myworkspace --output json \
  | jq -r '.[].links.clone[] | select(.name=="ssh") | .href'
```

List open pull request titles:

```bash
bitbucket pr list myworkspace/myrepo --state open --output json \
  | jq -r '.[].title'
```

Fail a CI step when a PR has unresolved tasks:

```bash
count=$(bitbucket pr task list myworkspace/myrepo 42 --output json \
  | jq '[.[] | select(.state=="UNRESOLVED")] | length')
[ "$count" -eq 0 ] || { echo "$count unresolved task(s)"; exit 1; }
```
