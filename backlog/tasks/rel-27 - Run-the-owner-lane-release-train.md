---
id: REL-27
title: Run the owner-lane release train
status: In Progress
assignee: []
created_date: '2026-10-24 15:57'
updated_date: '2026-10-24 16:19'
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
- [x] #7 A push to main tags nothing: it reports the version a release would get and stops, so merges spend no version space (decision-14 §9).
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Measured, 2026-10-24 — run 40109716135 (event push, main, 407f323, the merge of #34). Plan step: version 0.1.0 from Convco, action=tag, because no v* tag exists. Gates step: passed (108 tests, actionlint, workflow-check, repo-check). `pixi run --frozen package`: failed in one second, before any solve — `Failed to render recipe with variants` / `Jinja template error: undefined value: No stdlib found for language: c (template: ${{ stdlib("c") }})`. The tag step never ran and `git ls-remote --tags origin` is empty, so no version was consumed.

Two corrections came out of that run, both in this task s PR:

1. Packaging. recipe.yaml renders stdlib(c), but rattler-build defaults only the compiler per platform — the C standard library must come from the variant configuration. `recipe/variants.yaml` now supplies `c_stdlib: sysroot` and `c_stdlib_version: 2.17` (conda-forge baseline, so the artifact records __glibc >=2.17 instead of inheriting the runner glibc). Auto-discovered beside the recipe, so the `package` task keeps one flag-free command. Verified only to the render boundary: conda-forge is unreachable from the development sandbox, so `rattler-build` cannot run past the template here.

2. Trigger. A merge was cutting versions, which made the milestone tag redundant and spent version space on how often main moved. decision-14 §9 now separates the two events: a `main` push observes (computes the version, runs nothing else), while a milestone tag or a `workflow_dispatch` asks (gates → packaging proof → notes → tag → dispatch release.yml). Implemented in autorelease.yml and exercised by replaying the plan step off the file for six cases: push/branch → observe; push/milestone tag → tag; dispatch → tag; dispatch + suffix rc.1 + dry-run → tag v0.1.0-rc.1, dry_run=true; push with the tag present → skip; bump=sideways → refused with exit 2.

AC#2..#5 remain open: no dispatched run has completed yet, and the artifact has still never been built by CI. AC#6 is proven by the merge commit that lands this task.

Observe path measured, 2026-10-24 — run 40110923823 (event push, main, 98f956f, the merge of #36). action=observe, version 0.1.0, no gates/package/notes steps ran, step summary: "autorelease: a merge spends no version space; dispatch this workflow or push a milestone tag to release v0.1.0 (decision-14 §9)", `git ls-remote --tags origin` still empty, run took 35 s.
<!-- SECTION:NOTES:END -->
