# Live-API Smoke Tests

Most of the CLI is covered by unit tests, but those assert request/response
*shapes* against what we believe the Bitbucket Cloud REST API v2.0 expects —
not against a live instance. The mutating endpoints below were implemented
from documentation and have **not** been exercised against a real workspace.
Run this checklist against a scratch repository before trusting them, and tick
each item once confirmed. If one fails, the note names the most likely cause.

> This file exists so that "known-unverified" is distinguishable from a
> regression when the first bug report arrives. Do not delete an item; mark it
> verified with the date and CLI version instead.

## Write paths pending live verification

- [ ] `repo reviewer add <repo> <account>` — `PUT .../default-reviewers/{target}`
      with an empty `{}` body. Uncertainty: the `{target}` identifier may need
      to be an account ID / `{uuid}` rather than a username (GDPR migration).
- [ ] `repo permission grant-user <repo> <user> --permission write` —
      `PUT .../permissions-config/users/{selected}` with `{"permission": …}`.
      Confirm the `permissions-config` path spelling and that `{selected}` is an
      account ID.
- [ ] `repo permission grant-group <repo> <group> --permission read` — same,
      `.../permissions-config/groups/{slug}`.
- [ ] `issue attachment add <repo> <id> <file>` — multipart `POST .../issues/{id}/attachments`.
      The multipart field name is `files`; Bitbucket derives the attachment name
      from the part filename, so this should be fine, but confirm the upload
      succeeds and the file appears.
- [ ] `repo branch-restriction create <repo> --kind … --pattern …` —
      `POST .../branch-restrictions`. Some kinds require a non-empty `users`/
      `groups` list (now settable via `--user`/`--group`); confirm the body is
      accepted for the kind you use.
- [ ] `repo environment create <repo> <name> --type Production` —
      `POST .../environments/` with nested `environment_type`. Some accounts may
      require a `rank` field.
- [ ] `pipeline schedule create <repo> --branch … --cron …` —
      `POST .../pipelines_config/schedules/`. Confirm the `selector.type`
      (`branches`) is accepted for your pipeline configuration.
- [ ] `pipeline variable set <repo> …` and `pipeline workspace-variable set …`
      — the repo path is `.../pipelines_config/variables/` (underscore) and the
      workspace path is `.../pipelines-config/variables/` (hyphen), both with a
      trailing slash. Confirm the create POST is accepted (a missing trailing
      slash would redirect the POST to a GET and silently not create).
- [ ] `pipeline cache list/delete <repo>` — `.../pipelines-config/caches/`
      (hyphen). Confirm the path returns 200, not 404.
- [ ] `pipeline config <repo> --next-build-number N` —
      `PUT .../pipelines_config/build_number` with `{"next": N}`.
- [ ] `pr create/edit --reviewers <account>` — reviewers serialize as
      `{"uuid": …}` / `{"username": …}`. Confirm reviewers are actually attached
      (GDPR-migrated Bitbucket ignores `username`).
