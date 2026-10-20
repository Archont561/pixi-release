---
id: REL-8
title: Generate the changelog through Convco
status: To Do
assignee: []
created_date: '2026-10-20 19:12'
updated_date: '2026-10-20 20:11'
labels:
  - release
milestone: M1
dependencies:
  - REL-2
priority: high
ordinal: 8000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Convco generates CHANGELOG.md for the planned version (decision-6). pixi-release stages that file and writes no release-note text.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 prepare stages the CHANGELOG.md that Convco generates for the planned version
- [ ] #2 pixi-release writes no release-note text of its own
- [ ] #3 prepare refuses when the changelog path is not listed in [version].changelog
- [ ] #4 Tests use rstest fixtures and proptest properties, written test-first at an agreed seam per the tdd skill (decision-11)
<!-- AC:END -->
