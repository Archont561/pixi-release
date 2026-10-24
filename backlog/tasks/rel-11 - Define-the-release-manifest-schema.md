---
id: REL-11
title: Define the release manifest schema
status: To Do
assignee: []
created_date: '2026-10-20 19:12'
updated_date: '2026-10-24 18:57'
labels:
  - architecture
milestone: m-2
dependencies:
  - REL-4
documentation:
  - >-
    backlog/decisions/decision-16 -
    Version-checking-is-driven-by-an-explicit-format-and-scheme-not-by-guessing.md
  - >-
    backlog/docs/plan/doc-11 -
    Development-plan-version-truth-evidence-then-surface.md
priority: high
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

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Unblocked, 2026-10-24 — decision-16 is accepted, including the detection clause (a closed table resolves format, key and scheme for a bare path, refusing an unrecognised file instead of guessing) and the environment-source clause (one policy-named variable that may only agree; an override stays the explicit authority). Schema 2 now has a settled target: REL-11 defines the table shape, REL-28 builds the checker over it, and REL-34 adds detection and markers on top of the same table.
<!-- SECTION:NOTES:END -->
