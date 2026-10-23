---
id: REL-17
title: Decide whether release-adjacent governance templates belong to pixi-release
status: To Do
assignee: []
created_date: '2026-10-23 15:01'
labels:
  - architecture
  - ci
milestone: m-4
dependencies: []
references:
  - 'https://github.com/Archont561/pixi-release/issues/9'
priority: medium
type: spike
ordinal: 17000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Issue #9 asks for generated issue forms with workspace scopes, a PR template, and a semantic PR-title workflow. decision-10 §2 confines the renderer to the release lane and doc-3 §2.2 owns only `release.yml` and `auto-release.yml`, so admitting these needs a decision, not a generator. The decision also fixes the single scope source: the Convco configuration, because decision-6 makes it the only version and changelog authority.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A decision records each proposed artifact (issue forms, issue `config.yml`, PR template, PR-title check) as admitted or rejected, with the reason and the lane it belongs to.
- [ ] #2 The scope list is single-sourced from `.versionrc` and Convco, with no second enumeration of scopes in any generated file (decision-6).
- [ ] #3 The decision-3 rendering rules (marker, stamp, policy digest, drift gate) are applied to each admitted file or waived explicitly.
- [ ] #4 decision-10 is confirmed or amended by the recorded decision, whichever the outcome requires, in the same change.
- [ ] #5 No generator code is merged under this task.
<!-- AC:END -->
