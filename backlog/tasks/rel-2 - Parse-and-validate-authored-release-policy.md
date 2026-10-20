---
id: REL-2
title: Parse and validate authored release policy
status: To Do
assignee: []
created_date: '2026-10-01 00:27'
updated_date: '2026-10-20 20:11'
labels:
  - release
  - architecture
milestone: m-1
dependencies:
  - REL-1
documentation:
  - backlog/docs/architecture/doc-1 - Release-controller-architecture.md
priority: high
type: feature
ordinal: 2000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Load `.pixi-release.toml` into typed configuration and reject ambiguous or unsafe policy combinations before any release command performs I/O. Preserve Convco files and make Convco the version/changelog authority when configured.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Configuration validation names the exact invalid key or incompatible authority choice.
- [ ] #2 Convco mode records the Convco version and configuration hashes as plan evidence.
- [ ] #3 Native SemVer behavior is rejected while active Convco policy is configured.
- [ ] #4 Proven by a green gates run on the merge commit that covers this task's tests
- [ ] #5 Tests use rstest fixtures and proptest properties, written test-first at an agreed seam per the tdd skill (decision-11)
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Configuration fixtures cover valid, incompatible, and missing-policy cases.
- [ ] #2 The public configuration contract is documented in Backlog architecture documentation.
<!-- DOD:END -->
