---
id: REL-10
title: Prove M0 with a green run of pixi run ci on main
status: To Do
assignee: []
created_date: '2026-10-20 19:12'
updated_date: '2026-10-23 15:20'
labels:
  - ci
milestone: m-0
dependencies:
  - REL-1
priority: high
ordinal: 10000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
M0 exit condition: pixi run ci succeeds from a fresh clone with only committed lockfiles, verified by CI on main. Blocked on this sandbox until the Actions billing or spending limit is resolved; the last run failed before any step started for that reason, not because of a test failure.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A fresh-clone CI run on main completes green
- [ ] #2 The Codecov upload step runs with CODECOV_TOKEN configured
<!-- AC:END -->
