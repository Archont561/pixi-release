# Agent sandbox environment

Facts about the **working environment**, not about the product. Nothing here is project
knowledge, so none of it has a Backlog home. ✅ measured 2026-10-04.

## Network

- Reachable: `api.github.com`, `registry.npmjs.org` (HTTP 200).
- **Not** reachable, both fail with `EOF`:
  - `uploads.github.com` — release asset upload (4/4 attempts).
  - `results-receiver.actions.githubusercontent.com` — so `gh run view --log` cannot work.

Consequence for any future probe: **never plan to read Actions logs or upload release assets
from here.** Have the workflow report back through an `api.github.com`-readable surface —
release body, PR comment, or check output.

## Toolchain

- Present: `node v22.22.3`, `npx 10.9.8`.
- Absent: `pixi`, `bun`, `cargo`.
- `AGENTS.md` mandates `pixi run backlog -- …`. Pixi is unavailable, so the honest equivalent is
  `npx -y backlog.md@1.53.0 …` — the exact version pinned in `package.json`.

## GitHub credentials

- `gh auth status` → authenticated as `Archont561` via a **GitHub App installation token**, not
  a PAT. Repository permissions `admin`/`maintain`/`push`/`pull`/`triage` all true.
- `gh api repos/.../actions/permissions` → `403 Resource not accessible by integration`. The
  token can still push workflow files and dispatch runs.

## CLI gotchas

- `gh release delete <tag>` and `GET /releases/tags/<tag>` do not work on **drafts**. Address a
  draft by numeric release ID, or list and filter on `tag_name`. (Why: `doc-5`.)
- `gh run list --limit 1` immediately after a push returns the *previous* run. Filter on a
  changed run id before trusting `status: completed`.
- `backlog decision` has **no `update` subcommand**, only `create` and `list`. Decision and
  document bodies are authored in the file after CLI creation.
