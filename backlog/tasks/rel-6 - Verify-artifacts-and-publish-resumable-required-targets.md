---
id: REL-6
title: Verify artifacts and publish resumable required targets
status: To Do
assignee: []
created_date: '2026-10-01 00:28'
updated_date: '2026-10-20 20:11'
labels:
  - release
  - testing
milestone: m-2
dependencies:
  - REL-4
  - REL-11
documentation:
  - backlog/docs/architecture/doc-1 - Release-controller-architecture.md
priority: high
type: feature
ordinal: 6000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement artifact manifests/checksums and the initial typed Prefix/Conda, GitHub Release, static Rust binary, and generic-command targets. Publish consumes verified evidence, tracks each target outcome, and resumes unfinished targets without repeating successful uploads.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Publication is impossible until verification evidence matches the immutable release plan.
- [ ] #2 A required target failure makes the release incomplete with resumable per-target state.
- [ ] #3 Dry runs and preview runners record commands without side effects.
- [ ] #4 Proven by a green gates run on the merge commit that covers this task's tests
- [ ] #5 Tests use rstest fixtures and proptest properties, written test-first at an agreed seam per the tdd skill (decision-11)
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Target fake tests prove verification-before-publish, required-target failure, and resume equivalence.
- [ ] #2 Artifact checksums and manifest schema are versioned JSON evidence.
<!-- DOD:END -->
