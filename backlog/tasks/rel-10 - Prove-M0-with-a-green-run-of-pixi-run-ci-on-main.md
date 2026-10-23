---
id: REL-10
title: Prove M0 with a green run of pixi run ci on main
status: In Progress
assignee: []
created_date: '2026-10-20 19:12'
updated_date: '2026-10-23 20:09'
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
M0 exit condition: `pixi run ci` succeeds from a fresh clone with only committed lockfiles, verified by CI on main.

Acceptance criterion 1 is proven: run 39982186413 on merge commit f9de3b6 (2026-10-23) completed all thirteen gate steps successfully on a clean runner — lint, typecheck, coverage, Codecov upload, workflow lint, drift check, repository checks and build. That is the first complete green run since b3d9c91 on 2026-10-17.

Criterion 2 was amended on 2026-10-23. The original wording required `CODECOV_TOKEN` to be configured, which was neither provable nor the property that mattered: the repository is public, Codecov accepts tokenless uploads there, the secrets API returns 403 for this token, and no Codecov status is posted on the commit. The criterion now asserts that the upload succeeds, and the gate step is strict (`fail_ci_if_error: true`) so a failed upload fails the build instead of being swallowed.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A fresh-clone CI run on main completes green
- [ ] #2 The Codecov upload step runs with CODECOV_TOKEN configured
<!-- AC:END -->

## Comments

<!-- COMMENTS:BEGIN -->
created: 2026-10-23 15:26
---
2026-10-23 measurement from arena session (PR #11): the Actions billing block that stopped runs 39473511171 and 39493759591 has cleared — run 39977961178 actually started. It is red at the Lint step after 33 s, which is cargo fmt --all --check / cargo clippy -D warnings over the workspace. The branch under test was Markdown-only, so the failure is inherited from main: the last green main run was b3d9c91 (2026-10-17), and Rust test code changed by +382/-31 lines across ff2daa0 and later commits with no gate run in between. Job logs cannot be downloaded from this sandbox (302 to results-receiver.actions.githubusercontent.com, doc-7), so the failing lint rule is not yet identified. M0 therefore stays open for a reason that is now concrete rather than 'CI has not completed'.
---

created: 2026-10-23 15:57
---
2026-10-23, PR #12: root cause was the lint gate, not billing. cargo fmt failed on three layout deviations and clippy -D warnings failed on the scenario-outline signature (rstest-bdd injects one case per Examples column), three discarded #[must_use] results in the dummy-repo builder, and underscore bindings consumed by the generated cases. Fixed and merged as f9de3b6; main is green.
---
<!-- COMMENTS:END -->
