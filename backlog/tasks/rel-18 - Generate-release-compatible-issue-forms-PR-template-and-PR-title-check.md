---
id: REL-18
title: 'Generate release-compatible issue forms, PR template and PR-title check'
status: To Do
assignee: []
created_date: '2026-10-23 15:01'
labels:
  - ci
  - testing
milestone: m-4
dependencies:
  - REL-17
  - REL-5
references:
  - 'https://github.com/Archont561/pixi-release/issues/9'
priority: low
type: feature
ordinal: 18000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Renders the governance files that REL-17 admits. Every rendered file goes through the existing renderer, the drift gate and the one command surface (decision-1, decision-3); no second CI path is added.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Generated issue forms are schema-valid and populate the scope dropdown from the single source named in REL-17.
- [ ] #2 The generated PR template carries the SemVer-impact and Backlog-link sections that REL-17 admits, and nothing outside them.
- [ ] #3 The PR-title check validates types and scopes against `.versionrc`, fails a non-conforming title, and is wired through the existing CI entry rather than a new workflow path (decision-1).
- [ ] #4 `pixi release workflow check` detects drift in every generated governance file (decision-3).
- [ ] #5 Proven by a green gates run on the merge commit that covers this task tests.
<!-- AC:END -->
