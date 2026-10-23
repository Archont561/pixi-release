---
id: REL-26
title: Package pixi-release as a conda artifact
status: In Progress
assignee: []
created_date: '2026-10-23 22:12'
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
- [ ] #1 A pixi task `package` builds the artifact described by `recipe/recipe.yaml` into `output/`, and `output/` is ignored by Git.
- [ ] #2 The recipe declares `convco` and `git` as run dependencies, pinned identically to `pixi.toml`, and `pixi run repo-check` fails when the recipe's version, package name or Convco pin drifts from the manifests that own them.
- [ ] #3 Packaging does not enter `gates` and adds no dependency to the default environment, so `pixi.lock` is unchanged and a packaging change cannot block a code review.
- [ ] #4 The recipe's own test block proves the run dependencies land: `pixi-release --version`, `convco --version` and `git --version` all run inside the built package's environment.
- [ ] #5 Proven by a green gates run on the merge commit that covers this task's tests.
<!-- AC:END -->
