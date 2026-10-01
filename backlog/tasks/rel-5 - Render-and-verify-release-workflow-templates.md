---
id: REL-5
title: Render and verify release workflow templates
status: To Do
assignee: []
created_date: '2026-10-01 00:28'
labels:
  - release
  - ci
  - testing
milestone: m-2
dependencies:
  - REL-3
documentation:
  - backlog/docs/architecture/doc-1 - Release-controller-architecture.md
priority: medium
type: feature
ordinal: 5000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Render GitHub Actions workflow YAML from typed policy and provide workflow check mode. Test through shared materialized fixtures, profile overlays, checked golden output, and negative temporary mutations rather than copied full repositories.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The renderer produces reviewed YAML from .pixi-release.toml without manually authored workflow drift.
- [ ] #2 workflow check detects generated-workflow drift and reports the source policy key.
- [ ] #3 Fixture materialisation excludes .git, .pixi, target, node_modules, dist, and prior release state.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Golden output and negative mutations run from a temporary materialized fixture.
- [ ] #2 Generated YAML passes actionlint in the canonical Pixi gate.
<!-- DOD:END -->
