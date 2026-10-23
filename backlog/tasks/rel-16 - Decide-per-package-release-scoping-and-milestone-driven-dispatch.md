---
id: REL-16
title: Decide per-package release scoping and milestone-driven dispatch
status: To Do
assignee: []
created_date: '2026-10-23 15:01'
labels:
  - release
  - architecture
milestone: m-3
dependencies:
  - REL-3
references:
  - 'https://github.com/Archont561/pixi-release/issues/8'
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
