---
id: REL-27
title: Run the owner-lane release train
status: In Progress
assignee: []
created_date: '2026-10-24 15:57'
labels:
  - release
  - ci
milestone: m-2
dependencies: []
priority: high
type: feature
ordinal: 27000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Both halves of the release lane from decision-14: autorelease.yml decides and tags, release.yml builds and publishes to the prefix.dev channel under the OIDC binding for environment prefix. Hand-authored owner-lane files, so repo-check is their drift gate until REL-5 renders the consumer shapes and REL-7 replaces the bootstrap tag step with a pinned prior controller.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 autorelease derives the version from Convco, refuses to re-release an existing tag, runs the gates and a packaging check before tagging, and dispatches release.yml after pushing the tag.
- [ ] #2 release stamps the tagged version into the manifests and the recipe, proves agreement with repo-check, and refuses to publish an artifact whose filename does not match the tag.
- [ ] #3 Publishing to channel archont561/archont561 uses GitHub OIDC under the prefix environment with id-token: write and no stored token; a prerelease tag skips the upload.
- [ ] #4 A GitHub Release carries the same artifact bytes plus SHA256SUMS.
- [ ] #5 One dispatched run proves the lane end to end, because YAML actionlint accepts has not thereby executed.
- [ ] #6 Proven by a green gates run on the merge commit that covers this task's tests.
<!-- AC:END -->
