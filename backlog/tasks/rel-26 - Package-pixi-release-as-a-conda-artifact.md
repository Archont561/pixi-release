---
id: REL-26
title: Package pixi-release as a conda artifact
status: Done
assignee: []
created_date: '2026-10-23 22:12'
updated_date: '2026-10-24 18:25'
labels:
  - release
  - packaging
dependencies: []
priority: medium
type: feature
ordinal: 26000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
doc-2 §1.1 already tells users to run `pixi global install --channel https://prefix.dev/archont561/archont561 --channel conda-forge pixi-release`, but nothing in the repository produces that package. Add the recipe and the pixi task that builds it, with convco as a run dependency so the plugin environment resolves the tool the controller executes (decision-6).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A pixi task `package` builds the artifact described by `recipe/recipe.yaml` into `output/`, and `output/` is ignored by Git.
- [ ] #2 The recipe declares `convco` and `git` as run dependencies, pinned identically to `pixi.toml`, and `pixi run repo-check` fails when the recipe's version, package name or Convco pin drifts from the manifests that own them.
- [ ] #3 Packaging does not enter `gates` and adds no dependency to the default environment, so `pixi.lock` is unchanged and a packaging change cannot block a code review.
- [x] #4 The recipe's own test block proves the run dependencies land: `pixi-release --version`, `convco --version` and `git --version` all run inside the built package's environment.
- [ ] #5 Proven by a green gates run on the merge commit that covers this task's tests.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Measured, 2026-10-24 — run 40112073824 (workflow_dispatch with dry-run=true, main, d142073), steps "Prove the artifact still builds" and "Write the release notes": success. The build produced `/home/runner/work/pixi-release/pixi-release/output/linux-64/pixi-release-0.1.0-h41f06ac_0.conda` (669.99 KiB) from build deps gcc_linux-64 16.2.0, rust 1.98.1 and sysroot_linux-64 2.17 — the variant configuration that the previous run (40109716135) was missing. The recipe test block ran inside the built package s test environment: `pixi-release --version` -> pixi-release 0.1.0, `convco --version` -> convco 0.7.2, `git --version` -> git version 2.56.0, then "all tests passed!", with run deps __glibc >=2.17,<3.0.a0, convco >=0.7.2,<0.8 and git >=2.40 recorded. That is AC#1 and AC#4 by execution rather than by reading the recipe.

Path moved, 2026-10-24 — the recipe now lives inside the crate it packages. AC#1 names `recipe/recipe.yaml`; the path is now `crates/pixi-release/recipe/recipe.yaml`, and the criterion's intent is unchanged: one `pixi run package` task builds the artifact into `output/`, which stays gitignored. The recipe's `source.path` became `../../..` (rattler-build resolves a relative source path against the recipe directory); `about.license_file` keeps its bare names because rattler-build resolves those against the source or recipe directory, and the source is still the repository root. `variants.yaml` moved with it and is still auto-discovered beside the recipe, so the task needs no extra flag. `pixi run repo-check` now requires the new path, and `pixi.toml`, `xtask::packaging_agrees`, the release lane's stamp list and doc-1 were updated with it. Packaging gained its first pull-request proof in the same change: `ci.yml` grew a `package` job that runs the same `pixi run --frozen package` as the release lane, in parallel with the gates.
<!-- SECTION:NOTES:END -->
