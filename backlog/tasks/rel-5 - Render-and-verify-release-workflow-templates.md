---
id: REL-5
title: Render and verify release workflow templates
status: To Do
assignee: []
created_date: '2026-10-01 00:28'
updated_date: '2026-10-20 20:11'
labels:
  - release
  - ci
  - testing
milestone: M1
dependencies:
  - REL-3
  - REL-9
documentation:
  - backlog/docs/architecture/doc-1 - Release-controller-architecture.md
priority: medium
type: feature
ordinal: 5000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Renderer half of the former REL-5: templates, golden fixtures, and the three consumer shapes. Drift gating moved to REL-12 (M2).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The renderer produces reviewed YAML from .pixi-release.toml without manually authored workflow drift.
- [ ] #2 Fixture materialisation excludes .git, .pixi, target, node_modules, dist, and prior release state.
- [ ] #3 Proven by a green gates run on the merge commit that covers this task's tests
- [ ] #4 Rendered workflows bootstrap pixi-release with pixi global install from https://prefix.dev/channels/@archont561/archont561, with no uses: line (decision-10)
- [ ] #5 Tests use rstest fixtures and proptest properties, written test-first at an agreed seam per the tdd skill (decision-11)
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Golden output and negative mutations run from a temporary materialized fixture.
- [ ] #2 Generated YAML passes actionlint in the canonical Pixi gate.
<!-- DOD:END -->
