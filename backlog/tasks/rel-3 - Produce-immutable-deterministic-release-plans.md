---
id: REL-3
title: Produce immutable deterministic release plans
status: In Progress
assignee: []
created_date: '2026-10-01 00:27'
updated_date: '2026-10-23 20:38'
labels:
  - release
  - testing
milestone: m-1
dependencies:
  - REL-2
documentation:
  - backlog/docs/architecture/doc-1 - Release-controller-architecture.md
priority: high
type: feature
ordinal: 3000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement `plan` as a read-only operation that derives a canonical JSON proposal, including selected version/tag, normalized targets, source revision, policy hashes, and external-tool evidence.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Identical repository and policy inputs produce byte-identical plan JSON.
- [x] #2 The plan identifies the version/tag authority and source revision it was derived from.
- [x] #3 plan performs no Git mutation, registry upload, or workflow write.
- [ ] #4 Proven by a green gates run on the merge commit that covers this task's tests
- [x] #5 Tests use rstest fixtures and proptest properties, written test-first at an agreed seam per the tdd skill (decision-11)
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Property tests cover target ordering, duplicate elimination, and deterministic serialization.
- [x] #2 A golden plan fixture is materialized from bounded fixture components.
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Seam: `pixi_release_core::plan` — `Plan::propose(&ReleasePolicy, PlanInputs, VersionAuthority)` is pure: the caller supplies the source revision, the `sha256:` policy digest and the tool version, and the module touches nothing. `normalize_targets` owns the canonical target rule; `Version`, `SourceRevision` and `PolicyDigest` validate their own inputs. The CLI (`pixi-release plan [--version V] [--bump …] [--out PATH] [--commit SHA]`) reads the policy and two Git facts, derives the version from Convco when one is not given, and writes nothing but the requested file. REL-2's `PlanEvidence` still owns the evidence-coherence rule and `Plan::propose` delegates to it, so the refusal has one implementation.

Measured proof: 24 cases in `crates/pixi-release-core/tests/plan.rs` (golden fixture, rstest cases, four proptest properties) and 8 in `crates/pixi-release/tests/cli.rs`, all written red first (`unresolved import pixi_release_core::plan`). `pixi run --frozen gates` exit 0 in 1m01s locally, 62 tests, `xtask check` passed.

AC#3 is proven by `plan_leaves_the_repository_untouched`: same `HEAD`, no tag, unchanged `git status --porcelain`, no `.github` directory, and the only output is the `--out` file. Contract recorded in `doc-1` §Plan format. AC#4 stays open until the merge commit's CI run is green.
<!-- SECTION:NOTES:END -->
