---
id: decision-7
title: One version per repository for v1
date: '2026-10-20 18:56'
status: accepted
---
## Context

The release specification (`doc-2`, `doc-3` §4) assumes one version per repository, one tag and one plan. Monorepos such as `geoaiworkbench` publish several packages, which would normally carry separate versions. Per-package versions change the tag format and the plan shape, so they are not a small addition.

## Decision

For v1 a repository has one version. Every manifest in `[version].manifests` carries that version, and one annotated tag marks the release. Per-package versions are out of scope for v1 and will need their own decision before implementation.

## Consequences

- The plan and the tag format stay single-valued, which keeps `decision-4` simple.
- Monorepos with independently versioned packages cannot use the v1 controller. They must wait for the per-package decision.
