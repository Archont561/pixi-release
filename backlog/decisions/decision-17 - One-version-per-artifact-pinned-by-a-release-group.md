---
id: decision-17
title: 'One version per artifact, pinned by a release group'
date: '2026-10-24 18:10'
status: proposed
---
## Context

`decision-7` fixes one version, one tag and one artifact for v1. For a single-artifact repository that
is correct, and `v0.1.0` of this repository is the proof.

Two sibling repositories have now hit the boundary of that rule from opposite directions.

- `qgis-sdk` holds a Python SDK and a TypeScript bridge. Under one shared version, `decision-24` made
  every release stamp and verify both sides; between `v0.1.1` and `0.2.0` the bridge had no source
  change at all, so that policy would publish a tarball identical to `0.1.1` and consume an immutable
  npm version to say nothing. `decision-25` superseded it: each package derives its own version from
  the commits that touch it, and a declared contract version ties the pair together.
- `pixi-sandbox` stays single-artifact and gets 13 assets per release — one artifact, many platforms.
  `doc-2` §2.4 already fixes the vocabulary: a target is a registry, not a `(registry, platform)` pair.

The distinction the current rule cannot express is between *platforms of one artifact* (which must
share a version, and do) and *several artifacts* (which must not).

## Decision

A repository declares a release group. A group names its artifacts, and each artifact derives and
carries its own version.

- An artifact is never re-released at a version it has already published (`decision-7`'s rule, per
  artifact).
- A release group records which artifact versions shipped together. That record is evidence, published
  with the release, and it is what a consumer reads to know a set is compatible.
- A contract version — a declared constant both sides read, verified on every release — is how a group
  stays coherent when its members move independently. It is checked, never derived.
- `pixi-release` itself stays single-artifact until a second artifact exists; the group is a
  first-class concept in the schema from the start, so adding one later is data, not a rewrite.

## Consequences

- `decision-7` is narrowed rather than deleted: one version per artifact, one tag per artifact, one
  artifact per publication.
- Tag grammar must express the artifact (`python-v0.2.0` vs `ts-v0.1.1`), so `project.tag-format`
  becomes per-artifact, and tag sorting must keep each artifact's base tag its own — the failure mode
  `qgis-sdk` fixed in `sort_tags(scope=…)`, where a `ts-v*` tag became Python's base.
- Scoped planning is required: a commit that touches no artifact plans nothing, and a tooling-only
  change (`scripts/`, `.github/`, `pixi.toml`) never manufactures a release. `qgis-sdk` learned this
  the hard way — its `detect_scope_from_files` counted every non-package path as `core` and cut
  `v0.2.0` from a tooling commit.
- Convco stays the version authority (`decision-6`); scoping filters the commit stream, it does not
  re-implement conventional-commit parsing a third time.
- REL-16 is the decision work; the schema work lands with REL-11, and no scoped release is possible
  before REL-6's evidence model.
