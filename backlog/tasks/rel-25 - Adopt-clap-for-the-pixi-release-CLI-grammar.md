---
id: REL-25
title: Adopt clap for the pixi-release CLI grammar
status: In Progress
assignee: []
created_date: '2026-10-23 21:51'
labels:
  - release
  - tooling
dependencies: []
priority: high
type: feature
ordinal: 25000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The CLI hand-parsed argv in four places. Replace that with a clap-derived grammar so one place owns subcommands, flags, usage text and exit codes, per decision-13. No command may gain a side effect.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The binary's grammar is derived by clap (decision-13), with no hand-written argv parsing left in crates/pixi-release.
- [ ] #2 Every pre-existing CLI behaviour is preserved: --version prints pixi-release <X.Y.Z>, usage errors exit 2, refusals exit 1, prepare still requires --dry-run.
- [ ] #3 Tests cover the grammar itself: help lists the lifecycle commands, an unknown flag and an unknown bump value are refused.
- [ ] #4 Proven by a green gates run on the merge commit that covers this task's tests.
<!-- AC:END -->
