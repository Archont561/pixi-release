---
id: doc-1
title: Release controller architecture
type: specification
created_date: '2026-10-01 00:27'
updated_date: '2026-10-23 22:14'
tags:
  - architecture
  - release
  - approved
---



# Release controller architecture

## Status

Approved project specification for the bootstrap boundary. Implementation tasks must link to this document and keep generated workflows, evidence formats, and adapters consistent with it.

## Product boundary

`pixi-release` is a typed release controller and workflow renderer for Pixi projects. It centralises release preparation, verification, artifact evidence, target publication, resume semantics, and generated workflow checks. It is not a generic CI generator and it must not duplicate Convco’s SemVer or changelog authority.

Audience boundary: the measured consumer population is `Archont561/*`, and that is the delivery target through v1. Specifications are written to public-product standard — nothing may assume "install my app" or org-private secrets (`decision-5`) — but public adoption is explicitly deferred to post-v1, and obligations that only a public audience creates (tier-2 monitoring of foreign registries, config migration tooling for strangers) land after it, not before.

## Authority and formats

- `.pixi-release.toml` is reviewed, authored release policy.
- Convco is the sole SemVer/changelog authority when `project.version-source = "convco"`; existing `.versionrc` and `.convco` are preserved.
- GitHub Actions workflows are generated YAML and are checked against the renderer.
- Immutable release plans, artifact manifests, and per-target resume evidence are JSON.

## Configuration contract

`doc-3` §4 owns the shape. This section records what the loader enforces, so a reviewer can tell a
policy refusal from a policy gap.

- Unknown keys are refused rather than ignored, so a typo cannot silently degrade to a default.
- Every refusal names the dotted key at fault (`schema`, `project.tag-format`, `version.manifests`,
  `target.name`, …) and cites the decision behind the rule. One document may yield several
  refusals at once; the first is the one the message leads with.
- `schema` must equal `1`.
- `project.version-source` accepts `convco` or `manifest`. `manifest` is refused while a Convco
  configuration (`.versionrc` or `.convco`) is present: Convco stays the single SemVer and
  changelog authority (`decision-6`).
- `project.tag-format` must contain `{version}`, because the tag is the evidence anchor
  (`decision-4`).
- `version.manifests` must be non-empty; `version.lockfiles` and `version.changelog` are required;
  no list may repeat a path or contain an empty one. `prepare` stages exactly what the policy
  names and nothing else.
- `gates.required`, when declared, must be non-empty: an empty list would let `cut` proceed with
  no proof.
- `workflows.render` admits only `auto-release` and `release` (`decision-10` §2); `ci` and `docs`
  stay hand-written.
- `target.name` admits only `github-release` before REL-6 (`decision-9`).
- A `convco` plan records the Convco version and the SHA-256 digest of every configuration file
  it read. Planning without that evidence is refused, and `manifest` mode refuses to carry it.

Validation performs no I/O. The caller supplies `PolicyContext { convco_configured }` and the
configuration digests, so the same document parses identically in a test and in a workflow, and
nothing is read from the network or the repository by the parser itself.

## Plan format

`plan` emits canonical JSON: the same inputs give the same bytes, and the golden fixture at
`crates/pixi-release-core/tests/fixtures/plan.golden.json` is the shape it must keep producing.
Each plan records:

- `schema`, `version` and `tag`, where the tag is `project.tag-format` with `{version}`
  substituted.
- `authority` — either `explicit`, where a human named the version, or `convco`, which carries the
  Convco version, the digests of the configuration it read and the requested `bump` (`decision-6`).
- `source` — the commit the plan was derived from and whether the worktree was clean.
- `policy.digest` — the `sha256:` digest of `.pixi-release.toml` at derivation time
  (`decision-3`).
- `targets` — one row per name, sorted, so the order a policy listed them in never changes the
  evidence.
- `tool` — the `pixi-release` version that produced the plan.

`plan` is pure with respect to the repository. It reads the policy and two Git facts and writes
only the `--out` path it was given; it never commits, tags, or writes a workflow.

## Prepare and cut

`prepare` reports the edits a release needs. Without `--dry-run` it refuses, because writing the
new literals is not implemented yet: it prints one `would update <path>: <old> -> <new>` line per
manifest named by `version.manifests` and names the changelog it would rewrite.

`cut` consumes a recorded plan and refuses before any mutation, in this order:

1. `branch` — a detached head is not a release branch.
2. `plan.source` — the plan came from another commit, or from a tree that was already dirty.
3. `policy.digest` — `.pixi-release.toml` changed since the plan was recorded.
4. `plan.touched` — the plan records no files.
5. `worktree` — a changed path the plan did not record, or nothing left to stage.
6. `tag` — the policy renders a different tag, or the tag already exists.
7. `gates.required` — a configured verb failed or never ran.

Only then does it stage exactly `plan.touched()`, commit `chore(release): <tag>`, create the
annotated tag with `pixi-release-version`, `plan-digest` and `policy-digest` trailers, and — only
with `--push` — push `refs/heads/<branch>` and then `refs/tags/<tag>`. `--dry-run` performs every
check and prints the operations instead of executing them.

The Git port has no publication operation, so `cut` cannot reach a registry or a GitHub release by
construction (`decision-9`). Verification and publication are separate commands.

## Lifecycle

The controller implements `plan → prepare → cut/tag → verify/build → publish → resume`. `cut` validates source state and never publishes by default. `publish` consumes verified recorded evidence, records each target outcome, and `resume` may continue only unfinished targets.

## Initial Rust topology

- `pixi-release-core`: deterministic release-domain types, state transitions, plan/manifest validation.
- `pixi-release-git`: semantic `GitProtocol`, stateful fake, then shell and local-bare-repository adapters.
- `pixi-release`: thin extension CLI whose grammar `clap` derives (`decision-13`), policy
  configuration load, command presentation, and workflow renderer entrypoints.

## Delivery mechanics

The controller is delivered as a conda package built from `crates/pixi-release/recipe/recipe.yaml`: `convco` is a run
dependency there because the binary executes it and never links it (`decision-6`), so
`pixi global install pixi-release` resolves the tool the controller spawns. The recipe is
authored, never generated, and its version/name/pins are checked by `repo-check` against the
manifests that own them.

Pixi owns the reproducible toolchain. The root Bun workspace owns `bun x turbo run <verb>` scripts; `crates/package.json` and `docs/package.json` provide package-local work; Turbo owns ordering. CI and Lefthook invoke `pixi run <verb>` only. The self-release workflow must install a checksum-verified pinned prior `pixi-release` release rather than controlling a release with the unreleased checkout binary.

## Validation model

Policy tests use pure domain code and fakes first. Add command recording, preview/no-side-effect runners, local-bare-Git integration tests, property tests for determinism/ordering/resume, and materialised shared fixtures with profile overlays. Fixtures are copied to temporary directories before mutation and exclude `.git`, `.pixi`, `target`, `node_modules`, `dist`, and prior release state.
