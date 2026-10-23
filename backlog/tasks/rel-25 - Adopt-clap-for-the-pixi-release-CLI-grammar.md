---
id: REL-25
title: Adopt clap for the pixi-release CLI grammar
status: Done
assignee: []
created_date: '2026-10-23 21:51'
updated_date: '2026-10-23 22:00'
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
- [x] #1 The binary's grammar is derived by clap (decision-13), with no hand-written argv parsing left in crates/pixi-release.
- [ ] #2 Every pre-existing CLI behaviour is preserved: --version prints pixi-release <X.Y.Z>, usage errors exit 2, refusals exit 1, prepare still requires --dry-run.
- [ ] #3 Tests cover the grammar itself: help lists the lifecycle commands, an unknown flag and an unknown bump value are refused.
- [ ] #4 Proven by a green gates run on the merge commit that covers this task's tests.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
`clap` 4.6.7 derives the grammar (`decision-13`). Hand-written parsing is gone: `Cli`, `Command` and one argument struct per subcommand replace the top-level match and the three flag loops.

Behaviour preserved, not re-specified: `--version` still prints `pixi-release 0.1.0`, usage errors exit 2, refusals exit 1, `prepare` still demands `--dry-run` (now a required flag, same message), and `plan --version` keeps meaning the explicit version because `propagate_version` stays off.

Measured: `pixi run --frozen gates` exit 0 with 104 tests — the 100 that existed plus four grammar tests. Fourteen pre-existing CLI tests passed unmodified.

Lock: the sandbox cannot reach crates.io, so the six crates were resolved against the vendored tree extended with GitHub-sourced copies at the published versions, and the lock carries the crates.io index checksums. CI on `09e9be0` downloaded those exact crates and `gates` passed, which verifies the lock; `ci` and `publish sandbox` are green on the merge commit.
<!-- SECTION:NOTES:END -->
