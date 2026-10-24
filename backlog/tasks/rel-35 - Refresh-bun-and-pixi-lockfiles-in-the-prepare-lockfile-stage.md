---
id: REL-35
title: Refresh bun and pixi lockfiles in the prepare lockfile stage
status: To Do
assignee: []
created_date: '2026-10-24 19:08'
labels:
  - release
milestone: m-1
dependencies:
  - REL-29
priority: medium
ordinal: 35000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
REL-29 gives prepare a lockfile stage, but only `Cargo.lock` has a refresher: `cargo update --workspace`. A policy that declares `bun.lock` or `pixi.lock` (doc-2 §, doc-3 §351, the xtask consumer fixtures) now gets an explicit `not refreshing … (REL-29)` line and the file stays out of the touched set, which is honest but incomplete: a release can still ship a lock that disagrees with the manifest it just stamped. Give the stage a refresher per format and make the refresh part of the same stage order, after the manifests agree.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 prepare refreshes a declared `bun.lock` by running `bun install --lockfile-only` after the manifest stage, and records it in the touched set.
- [ ] #2 prepare refreshes a declared `pixi.lock` by running `pixi lock` after the manifest stage, and records it in the touched set.
- [ ] #3 A declared lockfile with no refresher is refused by key rather than printed: the stage either refreshes a path or stops the prepare.
- [ ] #4 The refreshed tree passes `cargo --locked`, `bun install --frozen-lockfile` and `pixi install --frozen` with no further edit.
- [ ] #5 Proven by a green gates run on the merge commit, with rstest fixtures per format and a proptest over policy shapes (decision-11).
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Each refresher is a value in a table, not a match arm: adding a format cannot silently skip a path.
<!-- DOD:END -->
