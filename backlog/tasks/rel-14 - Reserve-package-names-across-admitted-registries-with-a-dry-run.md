---
id: REL-14
title: Reserve package names across admitted registries with a dry run
status: To Do
assignee: []
created_date: '2026-10-23 15:01'
labels:
  - release
  - testing
milestone: m-3
dependencies:
  - REL-13
  - REL-6
references:
  - 'https://github.com/Archont561/pixi-release/issues/8'
priority: medium
type: feature
ordinal: 14000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implementation half of issues #7 and #8. Builds, verifies and publishes 0.0.0 placeholder packages for the registries that REL-13 admits, following the policy and resume rules recorded there. Anything REL-13 rejects stays unimplemented, including the `--all` sweep over unadmitted targets.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 `pixi release reserve` accepts only registries admitted by REL-13 and refuses the rest with a diagnostic naming the registry and the decision that excludes it.
- [ ] #2 `--dry-run` builds and validates every archive and performs no network write; a test proves the request count is zero.
- [ ] #3 Missing credentials produce actionable diagnostics naming the registry, the variable or OIDC permission, and the bootstrap command that fixes it.
- [ ] #4 Each reservation produces evidence JSON with registry, name, 0.0.0 version, archive digests and a per-target outcome that resume reads without repeating a successful publish.
- [ ] #5 Tests use rstest fixtures and proptest properties, written test-first at an agreed seam per the tdd skill (decision-11).
- [ ] #6 Proven by a green gates run on the merge commit that covers this task tests.
<!-- AC:END -->
