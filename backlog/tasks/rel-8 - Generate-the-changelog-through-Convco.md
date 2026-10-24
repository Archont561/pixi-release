---
id: REL-8
title: Generate the changelog through Convco
status: Done
assignee: []
created_date: '2026-10-20 19:12'
updated_date: '2026-10-24 16:58'
labels:
  - release
milestone: m-1
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
- [x] #1 prepare stages the CHANGELOG.md that Convco generates for the planned version
- [x] #2 pixi-release writes no release-note text of its own
- [x] #3 prepare refuses when the changelog path is not listed in [version].changelog
- [x] #4 Tests use rstest fixtures and proptest properties, written test-first at an agreed seam per the tdd skill (decision-11)
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented 2026-10-24 at two seams agreed with the owner before the first test (tdd skill, decision-11): a pure core seam in `pixi_release_core::prepare` and the CLI seam through `prepare` in a dummy repository.

Core: `changelog_target(policy_changelog, requested)` answers with the policy s own path and refuses any other with the `version.changelog` key (AC#3); `stage(text, edit)` writes one literal using the same scan that reads it, and returns the text unchanged when the literal it was computed from is gone.

CLI: `prepare` now stages the planned literals and writes the bytes Convco prints for `convco changelog --unreleased <version>`. Convco is asked before anything is written, so a refusal leaves the worktree as it was. `--dry-run` reports the same work — including the changelog it would stage and its byte count — and writes nothing; `--changelog <path>` defaults to the policy value and is the only way to name a different one, which is then refused.

Evidence: 12 core tests (`prepare`, incl. the proptest properties `only_the_named_path_is_answered` and `staging_is_idempotent`) and 19 CLI tests pass locally under `pixi run --frozen gates` (lint, typecheck, test, lint-actions, workflow-check, repo-check) on the session branch. AC#2 is asserted as byte equality with Convco s own stdout, not as a snapshot.

Named deviation: the CLI refusal case (AC#3 second seam) was added after the flag existed, so it is an integration check of a rule that was red-first at the core seam, not a fresh red-green cycle.

Closed 2026-10-24: merge commit a9cc619 (PR #40) and the `ci` push run 40113826785 on it are green, which is the proof this task needs on the merge commit. m-1 now stands at 5/6 with REL-5 open.
<!-- SECTION:NOTES:END -->
