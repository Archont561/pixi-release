---
id: doc-1
title: Release controller architecture
type: specification
created_date: '2026-10-01 00:27'
updated_date: '2026-10-04 21:10'
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

## Lifecycle

The controller implements `plan → prepare → cut/tag → verify/build → publish → resume`. `cut` validates source state and never publishes by default. `publish` consumes verified recorded evidence, records each target outcome, and `resume` may continue only unfinished targets.

## Initial Rust topology

- `pixi-release-core`: deterministic release-domain types, state transitions, plan/manifest validation.
- `pixi-release-git`: semantic `GitProtocol`, stateful fake, then shell and local-bare-repository adapters.
- `pixi-release`: thin extension CLI, configuration load, command presentation, and workflow renderer entrypoints.

## Delivery mechanics

Pixi owns the reproducible toolchain. The root Bun workspace owns `bun x turbo run <verb>` scripts; `crates/package.json` and `docs/package.json` provide package-local work; Turbo owns ordering. CI and Lefthook invoke `pixi run <verb>` only. The self-release workflow must install a checksum-verified pinned prior `pixi-release` release rather than controlling a release with the unreleased checkout binary.

## Validation model

Policy tests use pure domain code and fakes first. Add command recording, preview/no-side-effect runners, local-bare-Git integration tests, property tests for determinism/ordering/resume, and materialised shared fixtures with profile overlays. Fixtures are copied to temporary directories before mutation and exclude `.git`, `.pixi`, `target`, `node_modules`, `dist`, and prior release state.
