---
id: REL-20
title: Sync release metadata into a documentation site
status: To Do
assignee: []
created_date: '2026-10-23 15:01'
labels:
  - release
  - testing
milestone: m-4
dependencies:
  - REL-19
  - REL-11
references:
  - 'https://github.com/Archont561/pixi-release/issues/10'
priority: low
type: feature
ordinal: 20000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implements the sync half of the contract REL-19 records: release version, changelog, download links and digests. It writes release facts into a site and never authors them.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Changelog content is copied from the Convco output and never authored by the tool (decision-6).
- [ ] #2 Version literals, download links and digests come from the release manifest, and only targets that succeeded appear (doc-8, decision-9).
- [ ] #3 Sync is idempotent: two runs against the same release produce byte-identical files.
- [ ] #4 Any generated deployment workflow carries the decision-3 marker and stamp and never floats a version pin.
- [ ] #5 Proven by a green gates run on the merge commit that covers this task tests.
<!-- AC:END -->
