---
id: REL-3
title: Produce immutable deterministic release plans
status: To Do
assignee: []
created_date: '2026-10-01 00:27'
labels:
  - release
  - testing
milestone: m-1
dependencies:
  - REL-2
documentation:
  - backlog/docs/architecture/doc-1 - Release-controller-architecture.md
priority: high
type: feature
ordinal: 3000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement `plan` as a read-only operation that derives a canonical JSON proposal, including selected version/tag, normalized targets, source revision, policy hashes, and external-tool evidence.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Identical repository and policy inputs produce byte-identical plan JSON.
- [ ] #2 The plan identifies the version/tag authority and source revision it was derived from.
- [ ] #3 plan performs no Git mutation, registry upload, or workflow write.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Property tests cover target ordering, duplicate elimination, and deterministic serialization.
- [ ] #2 A golden plan fixture is materialized from bounded fixture components.
<!-- DOD:END -->
