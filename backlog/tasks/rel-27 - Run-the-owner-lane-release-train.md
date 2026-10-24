---
id: REL-27
title: Run the owner-lane release train
status: In Progress
assignee: []
created_date: '2026-10-24 15:57'
updated_date: '2026-10-24 19:16'
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
- [x] #1 autorelease derives the version from Convco, refuses to re-release an existing tag, runs the gates and a packaging check before tagging, and dispatches release.yml after pushing the tag.
- [ ] #2 release verifies that the tagged tree already carries the tagged version — the three manifests and the recipe — refusing with the path that disagrees, and refuses to publish an artifact whose filename does not match the tag; it never writes a version into the tree after the tag exists.
- [x] #3 Publishing to channel archont561/archont561 uses GitHub OIDC under the prefix environment with id-token: write and no stored token; a prerelease tag skips the upload.
- [x] #4 A GitHub Release carries the same artifact bytes plus SHA256SUMS.
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

Measured, 2026-10-24 — the release was cut, and the lane ran end to end except its last job.

**The trigger.** `gh workflow run` is still HTTP 403 for this sandbox token, so the release was asked for the documented way: an annotated `milestone/sprint-1` tag pushed at 4ea8dc9. Run 40115046817 (push, milestone tag) succeeded in 2m10s: plan derived 0.1.0 from Convco, the gates ran, `pixi run package` built the artifact, Convco wrote the notes, `git tag -a v0.1.0` was pushed to origin, and `gh workflow run release.yml --ref v0.1.0` dispatched the lane. `refs/tags/v0.1.0` = bc9dc71, annotated, body = the Convco changelog. No version was re-released and no existing tag was overwritten (AC#1).

**The lane.** Run 40115209793: `build` succeeded in 50s (rattler-build loaded `recipe/variants.yaml`, resolved rust 1.98.1 / gcc_linux-64 16.2.0 / sysroot_linux-64 2.17, `xtask check: repository checks passed`, artifact `pixi-release-0.1.0-h41f06ac_0.conda`), and `publish to prefix.dev` succeeded in 31s — `pixi upload prefix --channel archont561/archont561 --generate-attestation --skip-existing`, with the OIDC guard step passing, so channel publication is proven with no stored token (AC#3).

**The failure.** `github release` failed in 6s: `failed to run git: fatal: not a git repository (or any of the parent directories): .git`. That job has no checkout — it only downloads artifacts — so `gh release create` had no repository to resolve. Fixed by passing `--repo "${GITHUB_REPOSITORY}"`. AC#4 stays open until a GitHub Release exists with the artifact and SHA256SUMS, and AC#5 stays open until the lane completes in one run.

**Still not exercised by this run:** the version stamp was a no-op (the manifests already said 0.1.0, which is what the tag says), and the filename-mismatch guard had no negative case. AC#2 stays open until a run stamps a version that differs from the manifests.

Repair path, 2026-10-24 — the last job can now be finished without rebuilding.

`release.yml` gained a `workflow_dispatch` input `existing_run_id`. When it is set, `build` is skipped (`if: inputs.existing_run_id == ''`), `prefix` follows it into skipped, and `github-release` runs from `always()` on either shape: the two downloads become `gh run download <id> --repo … -n conda-dist -D output` and `-n release-meta`, a new "Resolve the release identity" step derives tag/version/prerelease from `tag` (blank tag or a non-release tag is an `::error::` + exit 1), a new guard refuses a run that holds no `pixi-release-<version>` artifact, and "Verify the checksums match the artifact" runs `sha256sum --check SHA256SUMS` before anything is published. The publish step keeps `--repo … --verify-tag` from the fix merged as a04f2ff (PR #43, gates 40115490809).

Why not just re-run: a re-run replays the workflow file from the run's own ref, so 40115209793 would re-read the broken job at v0.1.0. Why not a plain dispatch: that rebuilds, and a conda package is not byte-reproducible (timestamps), so tag v0.1.0 would carry two different binaries (decision-7).

Verified locally against the environment the steps would see: the identity script yields tag_name/version/prerelease for normal, prerelease, repair and repair-prerelease, and exits 1 for a blank tag and for `milestone-sprint-1`; the fetch guard accepts `pixi-release-0.1.0-h41f06ac_0.conda` and rejects `pixi-release-0.0.9-…` when VERSION is 0.1.0. Not yet exercised by a real run — `gh workflow run` is 403 for this token, so the first repair run is the owner's to start.

Release repaired, 2026-10-24 — `v0.1.0` now has a GitHub Release, and it did not cost a second build.

Run 40117871020 (`release`, `workflow_dispatch`, `main`, `tag=v0.1.0`, `existing_run_id=40115209793`): `build` skipped, `publish to prefix.dev` skipped, `github release` success in 8s. https://github.com/Archont561/pixi-release/releases/tag/v0.1.0 — draft=false, prerelease=false, body is the Convco changelog carried by the tag annotation, assets `pixi-release-0.1.0-h41f06ac_0.conda` (692158 bytes) and `SHA256SUMS` (118 bytes).

Byte identity is the run's own chain, not an assumption: both files came from 40115209793's `conda-dist` and `release-meta` artifacts — the run whose `publish to prefix.dev` job uploaded to the channel — the filename guard matched `pixi-release-0.1.0`, and `sha256sum --check SHA256SUMS` passed before `gh release create`. Tags are unchanged (`v0.1.0` → bc9dc71 → 4ea8dc9), so nothing was re-cut.

AC#4 is checked on that evidence. AC#5 stays open: this run exercised the last job, not the lane end to end. AC#2 stays open: no run has yet stamped a version that differs from the manifests.

Side evidence, same day: a `workflow_dispatch` of `autorelease` naming the existing version (40117631616) finished in 38s having pushed nothing — the `skip` branch, "a version is never re-released (decision-7)" — which is the first observation of that guard on the dispatch path.

Sequenced, 2026-10-24 — AC#2 now has an owner. The deficiency recorded above (the stamp step was a no-op on v0.1.0, and the filename guard never saw a negative case) cannot be fixed inside the release lane without changing where the version is written: a publish job must consume a tree that already agrees with its tag. REL-30 moves the write into prepare, stages exactly the reported set in commit, and tags the commit it just made, then deletes the stamp step from release.yml. AC#2 is expected to close against a run of that shape, not against another repair. Ordering: doc-11.

AC#2 re-scoped, 2026-10-24 (REL-30): the release lane no longer stamps a version into the tagged tree — a publish job that rewrites the tree it is publishing builds an artifact no tag describes. The step is now "Verify the tagged tree already carries the tagged version": `xtask check --tag <tag>` compares Cargo.toml, pixi.toml, package.json and the recipe's context.version to the tag and refuses with the path that disagrees. AC#2 therefore closes when a release run passes that step; it is no longer waiting for a run that stamps a version differing from the manifests, because no run writes one any more.
<!-- SECTION:NOTES:END -->
