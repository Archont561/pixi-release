---
id: REL-33
title: Reuse the gate evidence a commit already has instead of re-running gates
status: To Do
assignee: []
created_date: '2026-10-24 18:12'
updated_date: '2026-10-24 18:12'
labels:
  - release
  - ci
milestone: m-2
dependencies:
  - REL-3
documentation:
  - >-
    backlog/docs/plan/doc-11 -
    Development-plan-version-truth-evidence-then-surface.md
priority: medium
type: feature
ordinal: 33000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
A release re-executes the whole gate suite at a commit where ci already passed: gates take 2m17-2m48 on every lane run (ci 40116861234, release 40115209793, autorelease 40115046817). The saving is not a cache, it is evidence reuse: the plan reads the gate conclusion recorded for that exact SHA, records it in the plan as proof, and refuses when it is absent or attached to another commit. Same discipline decision-8 applies to binaries, applied to proof. Measure the baseline before and after in the task notes; caching and matrix parallelism come only after that number exists.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The plan records the gate evidence for its commit: the check name, the conclusion, and the SHA it was recorded against.
- [ ] #2 A release whose commit carries no successful gate evidence refuses before building, naming the missing check; it never falls back to re-running gates silently.
- [ ] #3 Evidence attached to a different SHA is refused, proved by a fixture with a stale run.
- [ ] #4 The measured wall-clock of the owner-lane run before and after is recorded in the task notes, from two run ids.
- [ ] #5 Proven by a green gates run on the merge commit covering these tests.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 reuse is a policy field with a default, so a repository that wants fresh gates can say so and the plan records which choice it made.
<!-- DOD:END -->
