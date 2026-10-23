---
id: REL-11
title: Define the release manifest schema
status: To Do
assignee: []
created_date: '2026-10-20 19:12'
updated_date: '2026-10-23 15:20'
labels:
  - architecture
milestone: m-2
dependencies:
  - REL-4
priority: medium
ordinal: 11000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The JSON manifest that the release presentation sidecar (doc-8) and install guidance read. Decide fields, schema version, digest fields, and which targets appear, before REL-6 writes its evidence records.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Schema document lists tag, version, per-platform artifacts with SHA-256, and succeeded targets only
- [ ] #2 Schema version field and compatibility rule are recorded
- [ ] #3 Decision recorded on where the manifest is stored
- [ ] #4 Tests use rstest fixtures and proptest properties, written test-first at an agreed seam per the tdd skill (decision-11)
<!-- AC:END -->
