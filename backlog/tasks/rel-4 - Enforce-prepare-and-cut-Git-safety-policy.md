---
id: REL-4
title: Enforce prepare and cut Git safety policy
status: In Progress
assignee: []
created_date: '2026-10-01 00:28'
updated_date: '2026-10-23 21:06'
labels:
  - release
  - testing
milestone: m-1
dependencies:
  - REL-3
documentation:
  - backlog/docs/architecture/doc-1 - Release-controller-architecture.md
priority: high
type: feature
ordinal: 4000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Use the semantic Git port to implement dry-run preparation and a separate cut operation. Cut must validate branch, clean tree, revision, version/tag, policy, and configured gates before a commit or tag is pushed.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 prepare --dry-run records intended changes and leaves the worktree untouched.
- [x] #2 cut refuses dirty trees, wrong branches, mismatched tags, stale plans, and failed gates.
- [x] #3 cut never invokes publication targets.
- [ ] #4 Proven by a green gates run on the merge commit that covers this task's tests
- [x] #5 Tests use rstest fixtures and proptest properties, written test-first at an agreed seam per the tdd skill (decision-11)
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Stateful FakeGit tests cover every refusal condition.
- [x] #2 A local-bare-repository adapter test proves expected push/tag behavior.
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
### REL-4 implementation

`pixi-release-git` owns the semantic port: `GitProtocol` exposes `head_sha`, `dirty_paths`,
`is_clean` (derived from the paths), `branch`, `tags`; `ReleaseGit` adds `stage`, `commit`, `tag`,
`push` and records every call in `operations()`. `FakeGit` is a stateful fake with failure
injection; `LocalGit` is a subprocess adapter. Neither adapter has a publication operation, so
AC#3 is structural rather than a discipline.

`pixi-release-core::prepare` derives `version_edits` from the policy without writing, and
`pixi-release-core::cut` splits `check_repository` (facts only: branch, plan source, policy digest,
touched set, worktree, tag) from `permit` (facts plus gates). Ordering is asserted, so a refusal
key is stable contract. `cut::cut` emits `Stage(sorted touched)`, `Commit`, `Tag`, and `Push` only
when asked.

Measured: `pixi run --frozen gates` exit 0, 16 test binaries, 100 tests
(core `cut` 17, core `prepare` 5, git `local_repo` 4, CLI 14). The local-bare-repository test
proves the tag lands on the commit and both pushes reach the bare remote.
<!-- SECTION:NOTES:END -->
