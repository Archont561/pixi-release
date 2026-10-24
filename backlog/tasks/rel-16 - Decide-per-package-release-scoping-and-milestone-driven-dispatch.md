---
id: REL-16
title: Decide per-package release scoping and milestone-driven dispatch
status: To Do
assignee: []
created_date: '2026-10-23 15:01'
updated_date: '2026-10-24 18:12'
labels:
  - release
  - architecture
milestone: m-3
dependencies:
  - REL-3
references:
  - 'https://github.com/Archont561/pixi-release/issues/8'
documentation:
  - >-
    backlog/decisions/decision-17 -
    One-version-per-artifact-pinned-by-a-release-group.md
  - >-
    backlog/docs/plan/doc-11 -
    Development-plan-version-truth-evidence-then-surface.md
priority: low
type: spike
ordinal: 16000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Issues #7 and #8 ask for path-scoped releases (`python-v1.0.0`, `rust-v1.0.0`) and milestone tag triggers (`milestone-v*`, `milestone/*`, `m-v*`). decision-7 fixes one version and one tag per repository for v1 and decision-4 anchors evidence to that single annotated tag, so neither can be implemented as asked. doc-9 already records per-package versions as needing their own decision. This task makes that decision and classifies milestone tags as either a release trigger or a quality gate.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A decision is recorded and decision-7 is marked superseded or explicitly reaffirmed before any scoped-tag code exists.
- [ ] #2 Tag format, plan shape and evidence anchor for a scoped release are specified, or the request is rejected with the reason recorded.
- [ ] #3 Milestone tag triggers are classified as a release trigger or a gate, and the interaction with the Convco version computation (decision-6) is stated.
- [ ] #4 Scope detection from changed paths is reconciled with the single-version rule, and no second version authority is introduced.
- [ ] #5 No implementation is merged under this task.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Partly pre-empted 2026-10-24 by `decision-14` §4: milestone tags (`milestone-v*`, `milestone/*`, `m-v*`) are now **dispatch triggers only** — they ask the train for a release, carry no version, and anchor no evidence; every release still produces exactly one `v*` annotated tag. That satisfies AC#3 as far as the trigger question goes.

Still open under this task: AC#1 (whether `decision-7` is reaffirmed or superseded), AC#2 (scoped tag format and evidence anchor), AC#4 (path-scoped detection) and AC#5 stands — the implementation merged in REL-27 is the owner lane, not scoped-release code.
<!-- SECTION:NOTES:END -->
