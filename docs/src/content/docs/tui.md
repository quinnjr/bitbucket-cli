---
title: Interactive TUI
description: Browse Bitbucket resources in a full-screen terminal UI.
---

Launch the interactive terminal UI for a visual way to browse and manage your Bitbucket resources:

```bash
bitbucket tui

# or scope it to a workspace
bitbucket tui --workspace myworkspace
```

## Keyboard shortcuts

| Key | Action |
| --- | --- |
| `q` | Quit |
| `1`–`5` | Switch views: Dashboard, Repos, PRs, Issues, Pipelines |
| `j` / `k` or `↑` / `↓` | Navigate the list |
| `Enter` | Select (opens the chosen view from the Dashboard) |
| `r` | Refresh the current view |

The TUI loads each view's data on demand and surfaces load failures instead of showing an empty list.
