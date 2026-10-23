---
id: REL-12
title: 'Gate generated-workflow drift with markers, stamps and workflow check'
status: To Do
assignee: []
created_date: '2026-10-20 19:12'
updated_date: '2026-10-23 15:20'
labels:
  - release
milestone: m-2
dependencies:
  - REL-5
priority: medium
ordinal: 12000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The drift half of the former REL-5. The renderer marks generated files, stamps the policy digest, and workflow check reports drift with the source policy key.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 workflow check detects generated-workflow drift and reports the source policy key
- [ ] #2 Generated files carry the marker and the policy digest stamp
- [ ] #3 Tests use rstest fixtures and proptest properties, written test-first at an agreed seam per the tdd skill (decision-11)
<!-- AC:END -->
