---
id: doc-4
title: Sibling Repository Delivery Survey
type: other
created_date: '2026-10-04 12:32'
---
## Purpose

The canonical record of how delivery is actually done across `Archont561/*` as of 2026-10-04.
It is the evidence base for `doc-2`, `doc-3` and decisions 1–4; nothing here is itself a project
rule.

**Method.** Repositories read at HEAD through the GitHub API — `pixi-sandbox`, `geoquery`,
`pathway`, `qgis-rs`, `nadir`, `geoaiworkbench`. Files were fetched individually via
`gh api repos/Archont561/<repo>/contents/<path>`; the repositories were not cloned. Older repos
(`NinjaODM`, `digital-terrain-hub`, `umwl-geology-qgis-plugin`, coursework repos) predate the
current house style and were not used as evidence.

**Limits.** This is a point-in-time read of default branches. Line counts and identical-file
claims are ✅ measured; characterisations of intent are ⚠️ inference from code and comments.

---

## 1. The house style is already converged

Every one of the six current repositories shares the same spine:

| Element | pixi-sandbox | geoquery | pathway | qgis-rs | nadir | geoaiworkbench |
|---|---|---|---|---|---|---|
| `pixi.toml` as toolchain + task root | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Bun workspace at root + committed `bun.lock` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `biome.json` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `backlog/` + `backlog/config.yml` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `.agents/skills/` (+ `skills-lock.json`) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `AGENTS.md` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `lefthook.yml` | ✅ | ✅ | ✅ | ✅ | — | ✅ |
| `.versionrc` (convco) | ✅ | ✅ | ✅ | ✅ | ✅ | — |
| `.devcontainer/` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| `.pixi-sandbox.toml` | ✅ | ✅ | ✅ | — | ✅ | ✅ |
| `turbo.json` | ❌ (deferred) | ✅ | ✅ | ✅ | — | ✅ (`python/turbo.json`) |
| `crates/package.json` Turbo façade | — | ✅ | ✅ | ✅ | — | n/a |
| Three lockfiles (`pixi` + `bun` + `Cargo`) | ✅ | ✅ | ✅ | ✅ | ✅ | `pixi`+`bun` |

`pixi-release` as it stands matches this spine. It is missing `.gitignore`, `.versionrc`,
`.devcontainer/`, `.agents/skills/`, `.github/`, `deny.toml`, `.pixi-sandbox.toml`, and the
`.pixi-release.toml` its own README links to.

## 2. Monorepo layout: three distinct shapes in use

**A. Single-root Cargo + thin JS (`pixi-sandbox`, `nadir`)**

```
crates/<name>/            Cargo workspace members
crates/xtask/             typed repository automation
docs/                     one Astro package, the entire bun workspace
pixi.toml                 owns every task; no turbo
```

`pixi-sandbox` deliberately has **no Turbo**. Backlog `decision-3` ("Defer Turbo until the JS
workspace graph earns it", status `deferred`, measured 2026-10-01) records three numeric
triggers for revisiting: ≥3 JS packages, a real cross-package import edge, and ≥60 s of
repeated JS CI work (or >10 s local feedback for unchanged packages). It also pre-specifies the
migration: Turbo as a root `package.json` devDependency pinned by `bun.lock`, never a conda
dependency; `bun x turbo`; remote caching off.

**B. Polyglot `apps/` + `packages/` + `crates/` + `python/` with Turbo (`geoquery`, `pathway`, `qgis-rs`)**

```
apps/docs/                Astro + Starlight site
packages/<name>/          TS packages
crates/<name>/            Cargo members
crates/package.json       Turbo façade: build/test/typecheck/lint/fmt -> cargo
python/<name>/            per-package pyproject.toml (+ per-package pixi.toml in geoaiworkbench)
turbo.json                ordering + cache policy
package.json              `all:*` scripts -> `turbo run <verb>`
```

`geoquery/pixi.toml` states the division explicitly: *"The package-specific work … no longer
lives here. Each package owns its own commands, in the manifest a reader of that package already
opens … Turbo owns the ordering and caching across those manifests."* What stays in `pixi.toml`
is only repo-global work: versions, changelog, actionlint, conda publish, sandbox transport,
and `setup`.

**C. Per-package `pixi.toml` (`geoaiworkbench`)**

`python/geoai-{bench,core,mcp}/` each carry their own `pixi.toml` + `package.json` +
`pyproject.toml`, with `python/turbo.json` fanning out. The only repo without a Cargo workspace.

**Observation:** the Turbo façade crate-package (`crates/package.json`) is the convergent trick
for making Cargo a Turbo node. Three repos use it; `pixi-release` already does too.

## 3. The one command surface

Universal rule, stated in every `AGENTS.md` and enforced by workflow shape:

```
pixi run <verb>  →  (optionally) bun run all:<verb>  →  turbo run <verb>  →  package script
```

CI steps and git hooks call **only** `pixi run <task>`. `pixi-sandbox/.github/workflows/ci.yml`
opens with: *"Every step in this file is ONE line: `pixi run <task>`. … a step that needs more
than one line is a task in `pixi.toml` … never inline bash here."* `pixi-sandbox` additionally
ships an escape marker (`# multiline-run-allowed`) for the rare justified exception, and a
repo-check (`crates/xtask/src/repo_checks/workflow_shape.rs`) that enforces it.

`lefthook` splits on wall-clock, not trust: `pre-commit` = formatters only, glob-filtered,
`stage_fixed: true`; `pre-push` = the gates that need a compile; `commit-msg` = `convco check
--from-stdin`.

## 4. Workflow inventory

| Workflow | pixi-sandbox | geoquery | pathway | qgis-rs | nadir | geoaiworkbench |
|---|---|---|---|---|---|---|
| `ci.yml` | 85 | 183 | 100 | 157 | 154 | 157 |
| `docs.yml` | 50 | 74 | 118 | 83 | — | 147 |
| `relock.yml` | 99 | 99 | 99 | 103 | — | 96 |
| `publish-sandbox.yml` | 140 | 292 | 292 | 312 | 59 | 292 |
| `release.yml` | 223 | 202 | — | 171 | — | — |
| `auto-release.yml` / `autorelease.yml` | 142 | 86 | — | 78 | — | — |
| `consumer-proof.yml` | 132 | — | — | — | — | — |
| `airlock.yml` | 302 | — | — | — | — | — |
| `bench.yml` | — | — | 153 | — | — | — |

(line counts)

### 4a. Two of these are already machine-generated

`relock.yml` and `publish-sandbox.yml` both open with:

```
# Generated by pixi-sandbox init. Regenerate this file instead of editing it.
# pixi-sandbox-version: 0.5.2
```

Hashes confirm it. `relock.yml` is **byte-identical** across `geoquery`, `pathway` and
`pixi-sandbox`; `qgis-rs` differs only by four lines (an added `cargo fetch --locked` guard) and
`geoaiworkbench` only by the absent Cargo handling. `publish-sandbox.yml` is byte-identical
between `pathway` and `geoaiworkbench`, and differs from `geoquery` by three lines (a
`--script-path` value).

Counter-examples that are the whole point:

- `nadir/publish-sandbox.yml` is **59 lines with no stamp** — an unmanaged, stale copy.
- `pixi-sandbox/publish-sandbox.yml` is 140 lines, marker-carrying but **unstamped**, and says
  so: *"This repository's publisher follows the checked-out source, including for planning.
  Consumer workflows generated by `init` intentionally remain release-based."* The renderer
  emits a distinct owner variant.

### 4b. Everything else is hand-written and divergent

`ci.yml`, `docs.yml`, `release.yml`, `autorelease.yml`, `airlock.yml`, `consumer-proof.yml`,
`bench.yml` carry no generated marker and no two are alike. This is the unmanaged surface.

## 5. Release behaviour: two incompatible implementations

### 5a. `pixi-sandbox` — typed, split, dispatch-aware

`auto-release.yml` (cut) and `release.yml` (build + publish) are deliberately separate so
*"the version-bump policy lives in one reviewable place."* Every step is a pinned `uses:` or a
one-line `pixi run -e package …`. The logic is a tested `xtask` crate:

```
crates/xtask/src/
  prepare_release.rs     bump + changelog + lock; prints vX.Y.Z; writes .release-touched
  commit_release.rs      stage exactly .release-touched, refuse unaccounted modifications,
                         refuse an existing remote tag, commit/tag/push as <tool>[bot]
  release_assets.rs      SHA256SUMS, re-read to prove completeness
  conda_platforms.rs     fail-closed: one package per declared platform
  workflow.rs            render the consumer workflow
  version.rs
  repo_checks/           badges, bash32, channel_drift, conda_manifest, mutable_refs,
                         release_tags, relock, stale_refs, workflow_shape
```

Release matrix: five targets on five native runners
(`x86_64/aarch64-unknown-linux-musl`, `x86_64/aarch64-apple-darwin`,
`x86_64-pc-windows-msvc`). The trick that makes this possible is an **empty pixi environment**:

```toml
[environments]
package = []   # resolves nowhere, so `pixi run -e package <task>` runs on win-64 too
```

Publishing uses GitHub OIDC (`id-token: write`, `environment: release`) to prefix.dev — no
long-lived `PREFIX_API_KEY`.

### 5b. `geoquery` / `qgis-rs` — inline bash

`autorelease.yml` does prepare+cut in a ~40-line shell block, including a **hand-maintained
file list**:

```bash
git add CHANGELOG.md Cargo.lock bun.lock pixi.lock \
  Cargo.toml pixi.toml package.json \
  apps/docs/package.json packages/geoquery/package.json \
  python/geoquery/pixi.toml python/geoquery/pyproject.toml
```

The band-aid is `pixi run version-check` (a `scripts/version.ts` that asserts every manifest
agrees), wired into `gates`. The list and the checker are two places that both have to be
updated when a package is added.

`geoquery/release.yml` publishes to npm and PyPI via OIDC Trusted Publishing; the version
derivation is `convco version --bump` inline.

## 6. Hard-won rules written into the YAML comments

These are failures the family has already paid for. Each is a candidate invariant.

1. **A `GITHUB_TOKEN` push or tag starts no workflow.** Every automated push therefore ends in
   an explicit `gh workflow run …` hand-off. `pixi-sandbox` dispatches four of them
   (`release.yml`, `consumer-proof.yml`, `docs.yml`, `publish-sandbox.yml`) and names the
   incidents: *"v0.4.2 exposed: the transport stood at dbe57a8/0.4.1 while main and the release
   were at 7745e9c/0.4.2"*; Pages kept serving the previous version; the relock bot's own commit
   never got a verdict.
2. **Job-level `permissions:` is a replacement, not an addition.** *"`contents: write` alone
   therefore pushes the lock commit and then fails the dispatch below with a 403, which is
   exactly how this was found."*
3. **Never interpolate `${{ github.head_ref }}` into a `run:` line** — pass it through `env:`.
   Script injection; actionlint rejects it.
4. **Refuse a fork PR loudly** rather than let a read-only token read as a flaky push.
5. **Fail closed on a silent matrix gap** — `xtask check-conda-platforms` exists because
   `fail-fast: false` means one platform producing nothing is invisible.
6. **Pixi arg parsing**: `pixi run <task> -- --target <triple>`; the `--` is load-bearing.
   *"which is exactly how v0.4.0's build died on all five runners."*
7. **One runtime, not per-OS shell dialects** — staging and checksums moved into Rust because
   *"per-platform shell is where the v0.3.6 release died."*
8. **Every `uses:` is a full commit SHA** with a `# vX.Y.Z` trailer, enforced by
   `repo_checks/mutable_refs.rs`.
9. `concurrency` is `cancel-in-progress: true` for `ci`, **`false`** for every release lane.
10. `--locked` / `--frozen-lockfile` everywhere; the relock bot is the only escape hatch.

## 7. `pixi-sandbox`'s `decision-4` (status `proposed`) — the generated-file contract

> Adopted, with one addition, as this repository's `decision-3`. Do not confuse the two
> numbers: `decision-4` below is **pixi-sandbox's**.

The most directly relevant prior art for `pixi-release`:

- The binary self-updates **only in the upgrade lane**; generated CI starts from its exact
  pinned binary and never asks for `latest`.
- Generated files carry an **ownership marker + version stamp**, making them self-describing.
- `init --check` is the **drift gate**: render fresh, compare, exit non-zero, write nothing.
- A scheduled/manual job runs the updated binary and, on drift, **opens a PR**. The upgrade unit
  is "self-update, then re-run `init`", never a hand-edited version constant.
- **Config is never rewritten.** An older `schema` is reported as a finding naming its migration
  path. *"The gate is the review, not the edit."*
- Floating production pins are explicitly rejected.

## 7b. Action SHA pins in use across the account

✅ Measured. The single canonical pin list; rendered workflows in `doc-2` use these verbatim.

```text
actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1          # v7.0.1
prefix-dev/setup-pixi@d3f436a425481402e6a95a1d1fc10331c708cd9e      # v0.10.2
actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a    # v7.0.1
actions/download-artifact@3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c  # v8.0.1
Swatinem/rust-cache@6323deb102c322ba6fcbdcafc7e3dddab59af2b6        # v2.9.2
softprops/action-gh-release@efb35369e0ad2afab669f228072c1b0d510eae64 # v3.0.3
stefanzweifel/git-auto-commit-action@4a55954c782fc1ea30b9056cd3e7a2b40ca8887d # v7.2.0
```

One addition **not yet observed in the account** (⚠️ resolved from the upstream release on
2026-10-04, adopted by `decision-5` for the optional consumer-owned token-minting step):

```text
actions/create-github-app-token@bcd2ba49218906704ab6c1aa796996da409d3eb1    # v3.2.0
```

## 7c. Annotated-tag trailer space is unused

✅ `gh api search/code "git tag -a" user:Archont561` → **0 hits**.
✅ `pixi-sandbox`'s `crates/xtask/src/commit_release.rs` calls `git.tag_annotated` with an
**empty message**.

The real estate `decision-4` claims is therefore genuinely free across the whole account.

## 8. Config-file precedent (`.pixi-sandbox.toml`)

```toml
schema = 1
branch_prefix = "sandbox"
cargo_vendor = true

[[bundle]]
name = "developer"
environments = ["default", "web"]
platforms = ["linux-64"]
```

Small, versioned by `schema`, heavily commented, reviewed as data. Declaring an entry *is* what
schedules its proof job.

## 9. Loose ends observed

- `geoquery/ci.yml` and `qgis-rs/ci.yml` pin `Archont561/pixi-sandbox/setup@da230c2…`, but
  **HEAD of `pixi-sandbox` contains no `setup/action.yml`** (no `action.yml` anywhere in the
  tree). The SHA pin keeps those jobs working, but the composite-action surface is not
  maintained at HEAD. Whether a consumer entrypoint should be a composite action, a `pixi global
  install`, or both is an open question for any sibling tool.
- `nadir` has drifted furthest: no `lefthook.yml`, no `relock.yml`, an unstamped 59-line
  `publish-sandbox.yml`, and decisions under `backlog/docs/decisions/` rather than
  `backlog/decisions/`.
- `qgis-rs/package.json` uses `"@biomejs/biome": "latest"` and `"@types/bun": "latest"` —
  floating devDependencies against the family's own pinning rule.
- Three different `.versionrc` dialects are in use: TOML (`geoquery`) vs YAML
  (`pathway`, `qgis-rs`, `pixi-sandbox`).
- Two decision-file conventions: `backlog/decisions/decision-N - Title.md` (pixi-sandbox,
  geoaiworkbench, pixi-release) vs `backlog/docs/decisions/NNN-title.md` (nadir).
