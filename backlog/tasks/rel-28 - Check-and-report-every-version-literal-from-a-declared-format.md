---
id: REL-28
title: Check and report every version literal from a declared format
status: To Do
assignee: []
created_date: '2026-10-24 18:11'
updated_date: '2026-10-24 18:12'
labels:
  - release
  - testing
milestone: m-2
dependencies:
  - REL-11
documentation:
  - >-
    backlog/docs/plan/doc-11 -
    Development-plan-version-truth-evidence-then-surface.md
priority: high
type: feature
ordinal: 28000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement decision-16's reader in pixi-release-core: a manifest declares its format (toml, json, yaml, go-tag), its key and its scheme, and one checker answers 'does every manifest agree with the planned version, and where does one not'. Ship it as a verb (`verify`) so a consumer repo and this one run the same check, then delete the owner-lane duplicates it replaces. Fixes two defects the current single-shape scan has: a YAML `version:` line is silently skipped (so recipe/recipe.yaml cannot be listed in version.manifests at all), and Version derives an Ord over its raw string, making 0.10.0 sort before 0.9.0.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 verify reads a TOML, a JSON and a YAML manifest from one declared table and reports agreement with the plan's version; a disagreeing file names its line and both values and exits 1.
- [ ] #2 A YAML manifest spelled version: "0.1.0" is read and stamped; listing recipe/recipe.yaml in version.manifests no longer refuses with 'carries no version literal to update'.
- [ ] #3 Version either orders by parsed components or stops deriving Ord; a test pins 0.10.0 > 0.9.0 or the absence of the impl, so no caller can inherit a lexicographic order.
- [ ] #4 A go-tag manifest is accepted with no literal when the module path's major suffix matches the tag's major, and refused when they disagree.
- [ ] #5 xtask check calls the core checker: versions_agree and packaging_agrees are deleted rather than kept as a second implementation of the same rule.
- [ ] #6 Proven by a green gates run on the merge commit covering these tests; fixtures are rstest and the ordering property is proptest (decision-11).
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 A fixture repository with one disagreeing manifest yields exactly one finding, naming its path, line and both values.
- [ ] #2 The same checker answers for this repository in CI, so the owner-lane copy is deleted rather than left to drift.
- [ ] #3 Every format is a typed reader with its own negative test; no format is detected from a file suffix at runtime.
<!-- DOD:END -->
