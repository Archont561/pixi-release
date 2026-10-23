---
id: REL-15
title: Render trusted-publishing bootstrap for admitted registry targets
status: To Do
assignee: []
created_date: '2026-10-23 15:01'
labels:
  - release
  - ci
  - testing
milestone: m-3
dependencies:
  - REL-13
  - REL-5
  - REL-12
references:
  - 'https://github.com/Archont561/pixi-release/issues/7'
priority: medium
type: feature
ordinal: 15000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Issues #7 and #8 ask for generated OIDC trusted publishing per registry. The workflow permission is already specified for the channel target (doc-2 §3.2 renders `id-token: write` with an `environment:` boundary), so what is new is the per-registry trust configuration and the per-target publish jobs, which exist only after REL-13 admits targets. Templates follow decision-3 like every other rendered file.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Rendered publish jobs grant `id-token: write` only for registries the REL-13 decision says trust OIDC, and keep the `environment:` trust boundary.
- [ ] #2 Generated output names the exact trust configuration each registry needs (publisher, workflow, environment) with the commands or settings path, cited from the REL-13 decision rather than restated.
- [ ] #3 Every generated file carries the decision-3 marker, version stamp and policy digest, and no production pin floats.
- [ ] #4 Golden fixtures cover at least one OIDC-only registry and one token registry.
- [ ] #5 Proven by a green gates run on the merge commit that covers this task tests.
<!-- AC:END -->
