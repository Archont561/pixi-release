---
id: REL-10
title: Prove M0 with a green run of pixi run ci on main
status: To Do
assignee: []
created_date: '2026-10-20 19:12'
updated_date: '2026-10-23 15:26'
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

## Comments

<!-- COMMENTS:BEGIN -->
created: 2026-10-23 15:26
---
2026-10-23 measurement from arena session (PR #11): the Actions billing block that stopped runs 39473511171 and 39493759591 has cleared — run 39977961178 actually started. It is red at the Lint step after 33 s, which is cargo fmt --all --check / cargo clippy -D warnings over the workspace. The branch under test was Markdown-only, so the failure is inherited from main: the last green main run was b3d9c91 (2026-10-17), and Rust test code changed by +382/-31 lines across ff2daa0 and later commits with no gate run in between. Job logs cannot be downloaded from this sandbox (302 to results-receiver.actions.githubusercontent.com, doc-7), so the failing lint rule is not yet identified. M0 therefore stays open for a reason that is now concrete rather than 'CI has not completed'.
---
<!-- COMMENTS:END -->
