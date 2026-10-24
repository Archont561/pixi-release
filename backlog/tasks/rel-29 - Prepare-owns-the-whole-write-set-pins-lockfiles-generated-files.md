---
id: REL-29
title: 'Prepare owns the whole write set: pins, lockfiles, generated files'
status: To Do
assignee: []
created_date: '2026-10-24 18:11'
updated_date: '2026-10-24 18:12'
labels:
  - release
  - testing
milestone: m-1
dependencies:
  - REL-8
documentation:
  - >-
    backlog/docs/plan/doc-11 -
    Development-plan-version-truth-evidence-then-surface.md
priority: high
type: feature
ordinal: 29000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
prepare stages the version literals and the changelog today, and nothing else. Make it own every place the version is written, in an order that cannot disagree with itself: manifests, then internal path-dependency pins, then generated files re-rendered from the stamped manifest, then lockfiles refreshed last. Each stage reports into the touched set the plan carries. This is the v0.5.3 skew in pixi-sandbox (a workflow committed a 0.5.2 stamp because the renderer was compiled before the bump) and the internal-pin gap in this repository (workspace deps pin members at a literal version the current scan never rewrites).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 prepare rewrites internal path-dependency pins (Cargo [workspace.dependencies] members) when their literal disagrees with the target version, and records them in the touched set.
- [ ] #2 prepare refreshes every declared lockfile after the manifests agree, and the refreshed tree builds under cargo --locked and pixi --frozen with no further edit.
- [ ] #3 Generated files are rendered only after the manifests are stamped, and the render reads the version from the manifest at runtime rather than from a compiled-in constant.
- [ ] #4 A dry run reports every would-be edit from every stage and leaves the worktree byte-identical (verified by comparing the tree hash before and after).
- [ ] #5 Proven by a green gates run on the merge commit covering these tests, with rstest fixtures and proptest properties (decision-11).
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Each stage has a refusal for a value it cannot read or write, naming the path and the key.
- [ ] #2 The stage order is asserted by a test, not by a comment: reordering the stages fails the suite.
<!-- DOD:END -->
