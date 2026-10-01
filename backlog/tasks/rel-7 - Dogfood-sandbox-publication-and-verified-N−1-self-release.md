---
id: REL-7
title: Dogfood sandbox publication and verified N−1 self-release
status: To Do
assignee: []
created_date: '2026-10-01 00:28'
labels:
  - release
  - ci
  - bootstrap
milestone: m-2
dependencies:
  - REL-5
  - REL-6
documentation:
  - backlog/docs/architecture/doc-1 - Release-controller-architecture.md
priority: high
type: feature
ordinal: 7000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add the project sandbox publication path and its CI validation, then make the release workflow install a pinned prior pixi-release asset whose installer and binaries are checksum-verified. Never control a release with the unreleased checkout binary or an unverified latest download.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The sandbox plan is validated by CI without publishing transport output.
- [ ] #2 Self-release pins a released controller version, asset URL, and installer digest.
- [ ] #3 The release workflow verifies controller assets before invoking any mutable release action.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 A negative test rejects latest, missing checksum, and checkout-binary controller configurations.
- [ ] #2 Public bootstrap documentation explains the N−1 compatibility contract.
<!-- DOD:END -->
