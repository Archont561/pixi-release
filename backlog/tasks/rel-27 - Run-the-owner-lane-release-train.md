---
id: REL-27
title: Run the owner-lane release train
status: In Progress
assignee: []
created_date: '2026-10-24 15:57'
updated_date: '2026-10-24 16:08'
labels:
  - release
  - ci
milestone: m-2
dependencies: []
priority: high
type: feature
ordinal: 27000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Both halves of the release lane from decision-14: autorelease.yml decides and tags, release.yml builds and publishes to the prefix.dev channel under the OIDC binding for environment prefix. Hand-authored owner-lane files, so repo-check is their drift gate until REL-5 renders the consumer shapes and REL-7 replaces the bootstrap tag step with a pinned prior controller.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 autorelease derives the version from Convco, refuses to re-release an existing tag, runs the gates and a packaging check before tagging, and dispatches release.yml after pushing the tag.
- [ ] #2 release stamps the tagged version into the manifests and the recipe, proves agreement with repo-check, and refuses to publish an artifact whose filename does not match the tag.
- [ ] #3 Publishing to channel archont561/archont561 uses GitHub OIDC under the prefix environment with id-token: write and no stored token; a prerelease tag skips the upload.
- [ ] #4 A GitHub Release carries the same artifact bytes plus SHA256SUMS.
- [ ] #5 One dispatched run proves the lane end to end, because YAML actionlint accepts has not thereby executed.
- [ ] #6 Proven by a green gates run on the merge commit that covers this task's tests.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Measured, 2026-10-24 — run 40109716135 (event push, main, 407f323, the merge of #34). The plan step resolved version 0.1.0 from Convco and action=tag, because no v* tag exists. The gates step passed (108 tests, actionlint, workflow-check, repo-check). `pixi run --frozen package` then failed in one second, before any solve: `Failed to render recipe with variants` / `Jinja template error: undefined value: No stdlib found for language: c (template: ${{ stdlib(\"c\") }})`. The tag step never ran, and `git ls-remote --tags origin` is empty, so no version was consumed and REL-7 still gets to make 0.1.0 the first self-release if that is wanted.

Cause and fix: recipe.yaml renders stdlib(c), but rattler-build has no default C standard library in its variant configuration — only the compiler has a platform default. `recipe/variants.yaml` now supplies `c_stdlib: sysroot` and `c_stdlib_version: 2.17` (conda-forge baseline; the artifact records __glibc >=2.17 instead of inheriting the runner glibc). Rattler-build auto-discovers `variants.yaml` beside the recipe, so the `package` task keeps its single flag-free command. Reproduced only as far as the render boundary: conda-forge is unreachable from the development sandbox, so the packaging proof is still CI-only. AC#5 stays open until a dispatched run finishes; AC#6 stays open until the merge commit of the fix has a green gates run.
<!-- SECTION:NOTES:END -->
