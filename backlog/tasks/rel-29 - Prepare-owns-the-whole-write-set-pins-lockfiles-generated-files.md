---
id: REL-29
title: 'Prepare owns the whole write set: pins, lockfiles, generated files'
status: In Progress
assignee: []
created_date: '2026-10-24 18:11'
updated_date: '2026-10-24 19:14'
labels:
  - release
  - testing
milestone: m-1
dependencies:
  - REL-8
documentation:
  - >-
    backlog/docs/plan/doc-11 -
    Development-plan-version-truth-evidence-then-surface.md
priority: high
type: feature
ordinal: 29000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
prepare stages the version literals and the changelog today, and nothing else. Make it own every place the version is written, in an order that cannot disagree with itself: manifests, then internal path-dependency pins, then generated files re-rendered from the stamped manifest, then lockfiles refreshed last. Each stage reports into the touched set the plan carries. This is the v0.5.3 skew in pixi-sandbox (a workflow committed a 0.5.2 stamp because the renderer was compiled before the bump) and the internal-pin gap in this repository (workspace deps pin members at a literal version the current scan never rewrites).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 prepare rewrites internal path-dependency pins (Cargo [workspace.dependencies] members) when their literal disagrees with the target version, and records them in the touched set.
- [ ] #2 prepare refreshes every declared lockfile after the manifests agree, and the refreshed tree builds under cargo --locked and pixi --frozen with no further edit.
- [ ] #3 Generated files are rendered only after the manifests are stamped, and the render reads the version from the manifest at runtime rather than from a compiled-in constant.
- [x] #4 A dry run reports every would-be edit from every stage and leaves the worktree byte-identical (verified by comparing the tree hash before and after).
- [x] #5 Proven by a green gates run on the merge commit covering these tests, with rstest fixtures and proptest properties (decision-11).
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Each stage has a refusal for a value it cannot read or write, naming the path and the key.
- [x] #2 The stage order is asserted by a test, not by a comment: reordering the stages fails the suite.
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Cargo half implemented on the session branch (PR: REL-29). Stage order is data: `STAGES = [Manifests, Pins, Generated, Lockfiles, Changelog]`, exposed as `stages()` and printed in that order. Pin stage: only inside a `[workspace.dependencies]` table of a file named `Cargo.toml`, only lines that carry `path` and a quoted `version`; an unreadable literal is refused as `version.manifests` naming `path:line`. Lockfile stage: `cargo update --workspace` (offline, then online) runs after the manifests agree and only when a `Cargo.toml` was edited; a declared lockfile with no refresher is printed as `not refreshing <path> … (REL-29)` and kept out of the touched set, so AC#2 is proven for `Cargo.lock` only and REL-35 owns the rest. AC#3 has no renderer to order yet: `Stage::Generated` is in `STAGES` and empty until REL-5 lands (blocked on decision-3).

Measured proof, this branch: `pixi run --frozen gates` exit 0 in a scratch clone at main `c0b4478` + the branch diff (log kept at /tmp, not committed). New tests — core: `pins_move_with_the_workspace_version`, `a_pin_is_only_a_path_dependency_that_states_a_version`, `a_non_cargo_manifest_contributes_no_pins`, `only_the_workspace_dependency_table_is_scanned`, `an_unreadable_pin_is_refused`, `a_pin_never_appears_without_a_path_and_a_literal`, `the_stage_order_is_the_one_that_cannot_contradict_itself`; CLI: `prepare_moves_the_pins_and_then_the_lockfile` (asserts the three stage lines in order and `cargo build --locked --offline` on the prepared tree), `prepare_names_a_lockfile_it_cannot_refresh`, and `prepare_dry_run_reports_edits_without_touching_the_worktree` now compares a tree digest, as AC#4 words it.

Merge proof: PR #50 → merge commit `303436d`; branch run `40123929958` success (gates + package), `main` push run `40124147568` success, `autorelease` `40124147582` success. AC#5 checked. REL-29 stays In Progress: AC#2 is Cargo-only (REL-35) and AC#3 waits for the renderer (REL-5, decision-3).
<!-- SECTION:NOTES:END -->
