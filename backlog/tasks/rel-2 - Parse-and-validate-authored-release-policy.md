---
id: REL-2
title: Parse and validate authored release policy
status: Done
assignee: []
created_date: '2026-10-01 00:27'
updated_date: '2026-10-23 19:39'
labels:
  - release
  - architecture
milestone: m-1
dependencies:
  - REL-1
documentation:
  - backlog/docs/architecture/doc-1 - Release-controller-architecture.md
priority: high
type: feature
ordinal: 2000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Load `.pixi-release.toml` into typed configuration and reject ambiguous or unsafe policy combinations before any release command performs I/O. Preserve Convco files and make Convco the version/changelog authority when configured.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Configuration validation names the exact invalid key or incompatible authority choice.
- [x] #2 Convco mode records the Convco version and configuration hashes as plan evidence.
- [x] #3 Native SemVer behavior is rejected while active Convco policy is configured.
- [x] #4 Proven by a green gates run on the merge commit that covers this task's tests
- [x] #5 Tests use rstest fixtures and proptest properties, written test-first at an agreed seam per the tdd skill (decision-11)
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Configuration fixtures cover valid, incompatible, and missing-policy cases.
- [x] #2 The public configuration contract is documented in Backlog architecture documentation.
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Seam: `pixi_release_core::config` — `ReleasePolicy::parse_with(text, &PolicyContext)` returns every fault with its dotted key; `PlanEvidence` records the Convco version and SHA-256 digests. The CLI stays thin: `pixi-release plan [--policy PATH] [--convco-version V]` only reads files and prints the evidence line.

Measured proof: 20 cases in `crates/pixi-release-core/tests/config.rs` (rstest + proptest + rstest-bdd outline) and 6 in `crates/pixi-release/tests/cli.rs`; `pixi run --frozen gates` exit 0 locally (11 test binaries green, xtask repository checks passed). Tests were written first and observed red (`unresolved import pixi_release_core::config`) before the module existed.

Contract documented in `doc-1` §Configuration contract, with `doc-3` §4 kept as the single canonical shape. AC#4 stays open until the merge commit's CI run is green.
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
created: 2026-10-23 19:39
---
AC#4 proven: merge commit 93de61a, `ci` run 40012670658 — gates success in 3m09s (19:36:07Z → 19:39:16Z). All five ACs and both DoD items now checked; the local gates run and the CLI/core measurements are recorded in the implementation notes.
---
<!-- COMMENTS:END -->
