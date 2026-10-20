---
id: doc-7
title: Agent sandbox environment
type: other
created_date: '2026-10-20 17:55'
---
## Purpose

Facts about the **working environment** the agent sandbox provides, and the CLI gotchas that follow.
These are not product rules. Measurements were taken on 2026-10-04 (✅ measured) unless tagged
otherwise.

**Reference environment.** All ✅ measurements were taken on 2026-10-04 against `Archont561/*`
default branches and GitHub-hosted runners; probe runs used `Archont561/pixi-release` on branch
`arena/01a10687-pixi-release`. Re-measure before relying on a toolchain fact: the sandbox image can
change between sessions.

## Network

- Reachable: `api.github.com`, `registry.npmjs.org` (HTTP 200).
- **Not** reachable, both fail with `EOF`:
  - `uploads.github.com` — release asset upload (4/4 attempts).
  - `results-receiver.actions.githubusercontent.com` — so `gh run view --log` cannot work.

Consequence for any future probe: **never plan to read Actions logs or upload release assets from
here.** Have the workflow report back through an `api.github.com`-readable surface — release body,
PR comment, or check output.

## Toolchain

- Present: `node v22.22.3`, `npx 10.9.8`.
- Absent: `pixi`, `bun`, `cargo`.
- `AGENTS.md` mandates `pixi run backlog -- …`. Pixi is unavailable, so the honest equivalent is
  `npx -y backlog.md@1.53.0 …` — the exact version pinned in `package.json`.

## GitHub credentials

- `gh auth status` → authenticated as `Archont561` via a **GitHub App installation token**, not a
  PAT. Repository permissions `admin`/`maintain`/`push`/`pull`/`triage` all true.
- `gh api repos/.../actions/permissions` → `403 Resource not accessible by integration`. The token
  can still push workflow files and dispatch runs.

## CLI gotchas

- `gh release delete <tag>` and `GET /releases/tags/<tag>` do not work on **drafts**. Address a
  draft by numeric release ID, or list and filter on `tag_name`. (Why: `doc-5`.)
- `gh run list --limit 1` immediately after a push returns the *previous* run. Filter on a changed
  run id before trusting `status: completed`.
- `backlog decision` has **no `update` subcommand**, only `create` and `list`. Decision and document
  bodies are authored in the file after CLI creation.

## Update 2026-10-20 (measured)

- ✅ Bun reaches the sandbox through npm: `@oven/bun-linux-x64` installs locally. The repo pins
  `bun@1.3.11` (`pixi.toml`: `>=1.3.11,<1.4`), and that version was installed and used. The
  `1.4.3` build is outside the pin and was not used.
- ✅ `bun install --frozen-lockfile`, `bun test crates/xtask/scripts` (3 pass), `bun run typecheck`
  and `bun run build` pass. Pixi is still absent, so these run through the underlying scripts
  directly, not through `pixi run`.
- ✅ `static.rust-lang.org` is unreachable. Rust cannot be installed locally, so `cargo` is not
  available and no Rust test has been compiled.
- ✅ PyPI is reachable, so PyYAML installs in a local venv. All 10 TOML and 8 YAML files in the
  repository parse. This checks syntax only.
- ✅ `Archont561/pixi-sandbox` has no session branch. Its branches are `main`, `sandbox/developer-linux-64`,
  `arena/eb67ebe1-pixi-sandbox` and dependabot branches.
- ✅ GitHub Actions does not start jobs on this account. Run `39473511171` on `main` (`97f8030`)
  failed with "The job was not started because recent account payments have failed or your
  spending limit needs to be increased." Every criterion that needs a green run stays open until
  billing is restored.
