---
id: REL-4
title: Enforce prepare and cut Git safety policy
status: To Do
assignee: []
created_date: '2026-10-01 00:28'
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
- [ ] #1 prepare --dry-run records intended changes and leaves the worktree untouched.
- [ ] #2 cut refuses dirty trees, wrong branches, mismatched tags, stale plans, and failed gates.
- [ ] #3 cut never invokes publication targets.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Stateful FakeGit tests cover every refusal condition.
- [ ] #2 A local-bare-repository adapter test proves expected push/tag behavior.
<!-- DOD:END -->
