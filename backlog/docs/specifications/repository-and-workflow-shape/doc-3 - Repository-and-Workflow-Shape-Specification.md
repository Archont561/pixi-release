---
id: doc-3
title: Repository and Workflow Shape Specification
type: specification
created_date: '2026-10-04 12:32'
---
## Purpose

The canonical answer to two questions about **this** repository:

1. **Monorepo shape** — layout, workspace and package manifests, task-graph topology, lockfile
   strategy.
2. **GitHub workflow management behaviour** — which workflows `pixi-release` owns, generates and
   manages, and how.

Evidence: `doc-4`. The consumer-facing counterpart is `doc-2`. Rules extracted from the
determinations below are `decision-1` (task-graph entrypoint), `decision-2` (controller
boundary), `decision-3` (generated-file contract) and `decision-4` (evidence anchor); where a
section restates one, the decision is the normative text.

Provenance: **✅ measured** against repositories at HEAD; **⚠️ reasoned** and wanting validation.

---

## 0. What the survey actually determines

Two of the six `Archont561` repositories' workflows are already machine-generated, stamped, and
near-byte-identical (`relock.yml`, `publish-sandbox.yml` — rendered by `pixi-sandbox init`). The
other five to seven per repo (`ci`, `docs`, `release`, `autorelease`, `bench`, `airlock`,
`consumer-proof`) are hand-written and no two are alike. Release orchestration specifically
exists in two mutually incompatible implementations: a tested Rust `xtask` in `pixi-sandbox`,
and a ~40-line inline bash block with a hand-maintained `git add` file list in `geoquery` and
`qgis-rs`.

**`pixi-release`'s market is exactly that second group.** `pixi-sandbox` proved the generated,
stamped, drift-gated workflow model on the two workflows it owns; it just has no mandate over
the release lane. The shape below is "take `pixi-sandbox`'s proven delivery contract, and make
the release lane the thing it governs."

This re-frames the existing `AGENTS.md` mission statement in a useful way. `pixi-release` is not
only a controller for *this* repository's release — it is a renderer whose output must be
correct for at least three different consumer shapes already in the wild (§3).

---

## 1. Monorepo shape

### 1.1 Determination — target tree

Status markers are the state of the tree as scaffolded on 2026-10-04.

```
pixi-release/
├── .agents/skills/                 # vendored skills + skills-lock.json   [STILL MISSING]
├── .devcontainer/                  #                                       [done]
├── .github/
│   ├── dependabot.yml              # actions + cargo only; locks go through relock  [done]
│   └── workflows/ci.yml            # owner lane — §2.1                     [done]
│                                   # release-* are RENDERED, not hand-written [pending REL-4]
├── backlog/                        # canonical tasks/milestones/decisions/docs
├── crates/
│   ├── package.json                # Turbo façade for Cargo, all members   [exists]
│   ├── turbo.json                  #                                       [exists]
│   ├── pixi-release-core/          # deterministic domain                  [exists]
│   ├── pixi-release-git/           # GitProtocol + FakeGit                 [exists]
│   ├── pixi-release/               # the `pixi release` extension binary   [exists]
│   └── xtask/                      # owner-lane repo automation — §1.3     [done]
├── apps/docs/                      # removed 2026-10-20 — was §1.4; no public docs home yet
├── crates/xtask/templates/workflows/*.tmpl  # golden renderer templates             [exists, placeholder]
├── crates/xtask/fixtures/consumers/ # three consumer shapes — §1.5          [done, inputs only]
├── .pixi-release.toml              # authored policy, `schema = 1`         [removed 2026-10-20; §4 still specifies it]
├── .pixi-sandbox.toml              # this repo as a pixi-sandbox consumer  [DEFERRED]
├── .versionrc                      # convco authority, YAML dialect        [done]
├── .gitignore · .gitattributes     #                                       [done]
├── deny.toml                       # cargo-deny policy                     [done, tool not yet in pixi.toml]
├── lefthook.yml · turbo.json · package.json · pixi.toml · Cargo.toml
└── pixi.lock · bun.lock · Cargo.lock
```

Two things are deliberately **not** scaffolded:

- **`.github/workflows/release-*.yml`.** They are the product's output. Hand-writing them now
  would create exactly the unstamped hand-maintained copy `decision-3` exists to prevent, and
  `nadir` is the example of where that ends. The owner lane cuts v0.1.0 through `xtask` (§1.3)
  until `workflow render` exists.
- **`.pixi-sandbox.toml`.** Adopting a second bot before this repository has settled how two
  renderers share one `.github/workflows/` is premature; that coexistence question is still
  open in §6.

### 1.2 Determination — keep the single-root Cargo workspace; grow crates on triggers, not up front

`doc-1` fixes the initial topology at three crates. Keep it. The survey's lesson from
`pixi-sandbox` (`decision-3`) is that structural tooling should be adopted against **written,
measurable triggers**, not anticipated need. Apply the same discipline to crate seams:

| Future crate | Trigger that earns it | Milestone |
|---|---|---|
| `pixi-release-config` | `.pixi-release.toml` grows a second schema version, or config parsing exceeds ~400 lines inside `-core` | REL-2 |
| `pixi-release-render` | The renderer owns ≥3 distinct workflow templates **and** golden-file tests outgrow `crates/pixi-release/tests/` | REL-5 |
| `pixi-release-forge` | A second publication target lands (a GitHub port and a registry port need one trait and two fakes) | REL-6 |

Until a trigger fires, the code lives in `pixi-release-core` behind a module boundary. A module
promoted to a crate is a mechanical move; a crate created early is a dependency edge nobody can
delete.

### 1.3 Determination — add `crates/xtask` now, and keep it strictly owner-lane

This is the chicken-and-egg crate. `pixi-release` cannot use `pixi-release` to cut
`pixi-release` v0.1.0, and `AGENTS.md` already forbids the alternative: *"Self-release must
install a pinned, verified prior release — not execute the unreleased checkout as its own
controller."*

So there are two consumers of release logic in this repository and they must not be confused:

- **`crates/xtask`** — bootstrap and repository self-checks. Mirrors
  `pixi-sandbox/crates/xtask`: `repo_checks/` (mutable action refs, version agreement, workflow
  shape, stale refs), release-asset staging, checksum generation. This is what cuts v0.1.0.
- **`crates/pixi-release`** — the product. Once it can verifiably cut a release, the owner lane
  migrates to `pixi release …` **driven by the pinned N−1 release binary**, and `xtask` shrinks
  to repo checks only.

Write the migration trigger down: *xtask's release subcommands are deleted when
`consumer-proof` has proven a pinned prior `pixi-release` cutting this repository's release
end-to-end.* Without a written trigger, `xtask` becomes a permanent second release authority —
precisely the "second CI command path" `AGENTS.md` forbids.

### 1.4 Determination — `docs/` → `apps/docs/`, Astro + Starlight

Three of four peer repos use `apps/docs/` with Astro + Starlight and publish to Pages. The
current `docs/` is a Bun script stub. Moving it now is cheap; moving it after the docs site
exists is a redirect problem. Adopt `geoquery`'s version wiring: the site reads the version from
the one manifest that owns it at build time (`astro.config.mjs` walks up to the root
`Cargo.toml`), so a release rewrites **zero** documentation lines.

### 1.5 Determination — `crates/xtask/fixtures/consumers/` is a first-class deliverable

`pixi-release` renders files into other people's repositories. The only honest test of that is a
materialised fixture repository per supported shape, copied to a temp dir before mutation,
excluding `.git`, `.pixi`, `target`, `node_modules`, `dist` (already specified in `doc-1`
§Validation model). Seed them from the three shapes in §3. `pixi-sandbox` carries exactly this
(`crates/pixi-sandbox/tests/fixtures/transport/`) and it is what lets its golden-workflow tests
mean anything.

### 1.6 Immediate gaps — closed

All four were closed by the 2026-10-04 scaffold.

1. ~~No `.gitignore` exists.~~ Added, enforcing the `AGENTS.md` never-commit list.
   `.gitattributes` added alongside it to pin `eol=lf` and mark the lockfiles generated.
2. ~~`.pixi-release.toml` does not exist~~ but `README.md` linked it and `doc-1` calls it the
   authored policy file. Added at `schema = 1` per §4, which also gives the parser its first
   fixture.
3. ~~`.versionrc` does not exist~~ but `AGENTS.md` names Convco the authority. Added in the
   YAML dialect (`pathway`/`qgis-rs`/`pixi-sandbox` — 3 of 4), not `geoquery`'s TOML.
4. ~~`backlog/decisions/decision-1` is accepted with an empty body.~~ §1.7 was promoted into it.

One gap remains open: `deny.toml` is committed but `cargo-deny` is not yet in `pixi.toml`, so
nothing runs it. Adding the dependency requires re-solving `pixi.lock`, which belongs in its
own reviewed change.

### 1.7 Determination — the task-graph entrypoint

**`decision-1` (accepted).** Pixi → Bun → Turbo is the single route; the renderer must not
assume Turbo; `turbo.json` stays minimal with `cache: false` on every Cargo-backed node.

Turbo is not earned by `pixi-sandbox`'s deferral triggers and is kept anyway for a
product-specific reason. That reasoning, and the two obligations that bound the exception, are
in the decision.

---

## 2. GitHub workflow management behaviour

### 2.1 Determination — two lanes, never one

| | **Owner lane** (`.github/workflows/` here) | **Consumer lane** (what the renderer emits) |
|---|---|---|
| Authored by | humans, reviewed | `pixi release workflow render` |
| Pinned to | checked-out source (bootstrap), then pinned N−1 release | an exact released `pixi-release` version |
| Marker | *"Owner lane — not rendered. See §2.1."* | `# Generated by pixi-release …` + `# pixi-release-version: X.Y.Z` |
| Drift gate | `xtask check-repository` | `pixi release workflow check` |

This is `pixi-sandbox`'s split, and its own `publish-sandbox.yml` documents why: *"This
repository's publisher follows the checked-out source, including for planning. Consumer
workflows generated by `init` intentionally remain release-based."* A tool that tests itself
only through its own generated output cannot detect that its generator is broken.

### 2.2 Determination — the workflow set to own, in order

| Workflow | Lane | Status | Notes |
|---|---|---|---|
| `ci.yml` | owner | **ship first** | one `pixi run <task>` per step; the step list is `pixi run gates` broken out |
| `relock.yml` | consumer-generated, by `pixi-sandbox` | adopt as-is | do not re-invent; it is byte-identical across 3 peers |
| `publish-sandbox.yml` | consumer-generated, by `pixi-sandbox` | adopt as-is | add `.pixi-sandbox.toml` |
| `docs.yml` | owner | after `apps/docs/` | Pages, `paths:`-filtered |
| `auto-release.yml` | owner → product | **the target** | `plan → prepare → cut`; one job, no inline bash |
| `release.yml` | owner → product | **the target** | `verify → build → publish`; 5-platform matrix |
| `consumer-proof.yml` | owner | before v1 | render into a clean fixture from the *published* release and run it |
| `workflow-drift.yml` | owner | with REL-5 | scheduled `workflow check`; opens a PR on drift |

`bench.yml` and `airlock.yml` are peer-specific and out of scope.

### 2.3 Determination — the nine behaviours the renderer must encode

These are the failures the family has already paid for (survey §6). Each is a renderer
responsibility, not a documentation note — the entire value proposition is that a consumer
cannot make these mistakes.

1. **Emit dispatch hand-offs automatically.** A `GITHUB_TOKEN` push or tag starts no workflow.
   Any rendered job that pushes a commit or a tag **must** emit the matching
   `gh workflow run …` step. `pixi-sandbox` shipped three separate bugs from this (stale
   transport at v0.4.2, Pages serving the previous version, the relock bot's own commit never
   getting a verdict). A renderer that knows the workflow graph can discharge this edge
   mechanically. **This is the single strongest argument for the product.**
2. **One line per step.** Rendered steps are a pinned `uses:` or exactly one
   `pixi run <task> …`. Multi-line `run:` requires an explicit allow-marker, and
   `workflow check` fails without it (`pixi-sandbox` precedent:
   `repo_checks/workflow_shape.rs` + `# multiline-run-allowed`).
3. **Full-SHA action pins with a `# vX.Y.Z` trailer**, verified at check time. The renderer owns
   the pin set; a consumer upgrade is a regenerate, not a hand edit.
4. **Job-level `permissions:` is a replacement.** The renderer computes the full scope set per
   job from what the job does (push → `contents: write`; dispatch → `actions: write`; OIDC
   publish → `id-token: write`; Pages → `pages: write`). `pixi-sandbox` found this through a
   403 in production.
5. **No `${{ }}` inside a `run:`.** Attacker-controlled refs reach commands through `env:` only.
   Non-negotiable; actionlint agrees.
6. **Fork PRs are refused loudly**, with a message naming the local command to run instead.
7. **Fail closed on a silent matrix gap.** `fail-fast: false` plus a per-platform artifact means
   a platform that produced nothing is invisible. Every rendered matrix gets its completeness
   assertion (`check-*-platforms` equivalent) in the collecting job.
8. **`concurrency`**: `cancel-in-progress: true` for `ci`; **`false`** with a repository-scoped
   group for every release lane. Never cancel a release midway.
9. **Lockfile discipline**: `--locked` / `--frozen-lockfile` in every rendered step; the relock
   bot is the only writer.

### 2.4 Determination — the release lane is split `cut` ≠ `publish`

Render two workflows, not one, matching the lifecycle `plan → prepare → cut/tag →
verify/build → publish → resume`:

- **`auto-release.yml`** — `workflow_dispatch` + `workflow_call`, inputs `bump` (auto | patch |
  minor | major), `version`, `dry-run`. Full history (`fetch-depth: 0`) because Convco counts
  commits. Ends by dispatching the build lane on the new tag.
- **`release.yml`** — `push: tags: v*.*.*` + `workflow_dispatch(version)`. Build matrix, then a
  single `environment: release` job that verifies evidence and publishes.

`AGENTS.md` already requires this (*"`cut` must never publish by default"*); the rendered shape
must make it structurally impossible rather than conventionally discouraged.

### 2.5 Determination — delete the hand-maintained `git add` list

The concrete thing this product removes from `geoquery` and `qgis-rs` is the hand-maintained
eleven-path `git add` list in their `autorelease.yml`, quoted in `doc-4` §5b, together with the
`version-check` script that exists only to catch it going stale.

Replace with `pixi-sandbox`'s proven two-step contract, promoted to typed product behaviour:

1. `prepare` writes every version literal **and records exactly which paths it touched** (the
   `.release-touched` precedent) into the immutable plan JSON.
2. `cut` stages exactly that recorded set, **refuses any unaccounted modification**, refuses an
   existing remote tag, then commits and tags as `pixi-release[bot]`.

The file set becomes derived data from `.pixi-release.toml`, not a list in a YAML file that
nobody updates when a package is added. `geoquery`'s `version-check` gate exists only because
the list and reality can disagree; under this contract they cannot.

### 2.6 Determination — resume is an artifact contract, not a flag

`publish` writes one JSON evidence record per target and uploads it as a workflow artifact
before the GitHub Release is created. `resume` downloads the record for the tag and retries
**only** targets without a success record. This makes the hardest requirement in `AGENTS.md`
(*"Publish must validate evidence and be resumable per target"*) testable in CI with a
deliberately failing target, instead of only observable during a real partial outage.

Corollary, from the survey: publish to the canonical package channel **before** creating the
GitHub Release, so the Release is never the record of an upload that did not happen.

### 2.7 Determination — target the 5-platform matrix and the empty-environment trick

`pixi.toml` here declares `platforms = ["linux-64"]`, which is right for the development
lockfile and wrong for a distributed binary. Adopt `pixi-sandbox`'s solution verbatim, because
it is the only one in the family that has shipped on Windows:

```toml
[environments]
default = { features = ["dev"], solve-group = "default" }
package = []   # empty: resolves on every runner, win-64 included
```

Release jobs use `pixi run -e package <task>` with `run-install: false`, against the Rust
toolchain the runner already ships. Targets: `x86_64-unknown-linux-musl`,
`aarch64-unknown-linux-musl`, `x86_64-apple-darwin`, `aarch64-apple-darwin`,
`x86_64-pc-windows-msvc`. Publish via GitHub OIDC (`id-token: write`), not a long-lived API key.

And the rule behind it: **no per-OS shell dialects.** Staging, checksums, and manifest checks
are Rust subcommands so one runtime covers five runners —
*"per-platform shell is where the v0.3.6 release died."*

### 2.8 Determination — the generated-file contract

**`decision-3` (proposed).** Ownership marker, version stamp, policy digest,
`pixi release workflow check` as the drift gate, upgrades as reviewed pull requests, and
production pins that never float.

Adopted from `pixi-sandbox`'s `decision-4` rather than re-derived; the one addition is the
policy digest. Evidence for why the contract is needed: `doc-4` §4a and §9.

---

## 3. The renderer's consumer matrix

The renderer is wrong if it works for only one of these. All three exist today.

| Shape | Example | What the renderer must not assume |
|---|---|---|
| Single-root Cargo, no Turbo, one JS package | `pixi-sandbox`, `nadir` | that `bun x turbo` exists |
| Polyglot `apps/` + `packages/` + `crates/` + `python/`, Turbo | `geoquery`, `pathway`, `qgis-rs` | that one `cargo build` is the whole build |
| Per-package `pixi.toml`, no Cargo workspace | `geoaiworkbench` | that a root `Cargo.toml` exists |

The invariant that survives all three is the one the family already agreed on: **`pixi run
<verb>` is the interface.** `.pixi-release.toml` names verbs and targets; it never names
`cargo`, `turbo`, `bun`, or `pytest`.

---

## 4. `.pixi-release.toml` — proposed skeleton

This is the **single canonical description of `.pixi-release.toml`**. `doc-2` renders workflows
from it and does not restate it.

Following the `.pixi-sandbox.toml` precedent: small, `schema`-versioned, reviewed as data,
declaring an entry is what schedules its proof.

```toml
schema = 1

[project]
version-source = "convco"      # convco | manifest ; convco stays authoritative (AGENTS.md)
tag-format     = "v{version}"
bot-identity   = "pixi-release[bot]"

# Every file whose version literal `prepare` owns. This replaces the hand-maintained
# `git add` list in geoquery/qgis-rs — `cut` stages exactly what `prepare` reports here
# and refuses anything else.
[version]
manifests = ["Cargo.toml", "pixi.toml", "package.json", "apps/docs/package.json"]
lockfiles = ["Cargo.lock", "bun.lock", "pixi.lock"]
changelog = "CHANGELOG.md"

# Verbs, never tools. How `pixi run gates` is implemented is the consumer's business.
[gates]
required = ["gates"]

# pixi-release never compiles anything (decision-2). The matrix is policy data: the controller
# copies these values into the workflow and runs YOUR task.
[build]
task        = "build-release-binary"   # your pixi task; the controller does not know it is cargo
environment = "package"                # pixi env; an empty env renders run-install: false

[[build.matrix]]
platform  = "linux-64"
runner    = "ubuntu-latest"
args      = "--target x86_64-unknown-linux-musl"
artifacts = ["target/x86_64-unknown-linux-musl/release/mytool"]
# …one entry per platform

# The publish job's approval gate. Deployment environments need GitHub Pro on private
# repositories (✅ measured 2026-10-04: this repository, private on a free plan, cannot
# create one). Omit `environment` and the publish job renders without a gate; `doctor`
# reports the missing gate as a finding rather than rendering a line the repo cannot satisfy.
[publish]
environment = "release"

[proof]
enabled              = true
isolated             = true        # env -i, scratch HOME, minimal PATH — no runner or repo state
smoke                = ["{asset} --version", "{asset} --help"]
expect-version       = true        # the binary's --version must equal the tag
deterministic-render = false       # true if your tool generates files

[workflows]
render  = ["ci", "auto-release", "release"]
runner  = "ubuntu-latest"

# A target is a registry, not a (registry, platform) pair — doc-2 §2.4.
[[target]]
name     = "github-release"
required = true

[[target]]
name     = "prefix"
channel  = "archont561/archont561"
auth     = "oidc"
required = true
platforms = ["linux-64", "linux-aarch64", "osx-64", "osx-arm64", "win-64"]
```

`AGENTS.md` gates this: *"Do not add a registry target until its artifact manifest and resume
behavior are specified and tested."* The `[[target]]` blocks above are illustrative of the
shape; only `github-release` is admissible before REL-6.

---

## 5. Sequenced consequences

Revised 2026-10-20. The task and milestone mapping lives in `doc-9` (Roadmap); this table names the
tasks each consequence now belongs to.

| # | Action | Milestone / task |
|---|---|---|
| 1 | Add `.gitignore`, `.versionrc`, `deny.toml` | M0 / REL-1 (done) |
| 2 | Write the body of `decision-1` | Done (accepted) |
| 3 | Add `.github/workflows/ci.yml`, one `pixi run <task>` per step | M0 / REL-1 (done); green run on main is REL-10 |
| 4 | `pixi-sandbox`'s relock and publish workflows | Out of scope for pixi-release; owned by `pixi-sandbox` |
| 5 | `crates/xtask` with `repo_checks/` and release bootstrap, with its deletion trigger | M0 / REL-1 (done); deletion after REL-7 |
| 6 | Ship the `.pixi-release.toml` skeleton; `-core` parses `schema = 1` | M1 / REL-2 |
| 7 | `prepare` records its touched set into the plan JSON; `cut` refuses anything else | M1 / REL-4 |
| 8 | `apps/docs/` with `docs.yml` | Removed 2026-10-20; no public-docs task yet |
| 9 | Renderer, golden templates, and `crates/xtask/fixtures/consumers/` for the three shapes | M1 / REL-5 (blocked by REL-9) |
| 10 | Marker, stamp, `workflow check` drift gate, `workflow-drift.yml` PR bot | M2 / REL-12 |
| 11 | Evidence JSON, per-target resume, five-platform matrix, OIDC publish | M2 / REL-6 (needs REL-11 manifest schema) |
| 12 | `consumer-proof.yml`; migrate the owner lane to the pinned N−1 release; shrink `xtask` | M2 / REL-7 |
| 13 | Convco changelog generation | M1 / REL-8 |

---

## 6. Open questions for the maintainer

Tracked by REL-9. These block M1 sizing until answered.

1. **Consumer entrypoint.** `geoquery` and `qgis-rs` pin `Archont561/pixi-sandbox/setup@<sha>`,
   but no `setup/action.yml` exists at `pixi-sandbox` HEAD. Should `pixi-release` ship a
   composite `setup/action.yml`, rely on `pixi global install` from a package channel, or both?
   This decides whether rendered workflows contain a `uses:` or a bootstrap step.
2. **Scope of the renderer.** Does `pixi-release` claim `ci.yml` and `docs.yml` (the remaining
   divergent hand-written workflows), or stay strictly in the release lane? §2.2 proposes
   claiming `ci` because the one-line-per-step rule is only enforceable if the renderer owns the
   file — but that is a materially larger product.
3. **Relationship to `pixi-sandbox`.** Both tools render workflows into the same
   `.github/workflows/` directory, both will carry ownership markers, both will want a drift
   PR bot. Do they share a marker/stamp convention and a single upgrade PR, or does each
   consumer repository get two bots?
4. **`pixi-sandbox`'s `xtask` as the seed.** `prepare_release.rs`, `commit_release.rs`,
   `release_assets.rs`, `conda_platforms.rs`, `workflow.rs` and `repo_checks/` are already
   tested against real git and bare remotes. Is `pixi-release` v0.1.0 a generalisation of that
   code (fastest path, proven behaviour), or a clean-room implementation against `doc-1`?
