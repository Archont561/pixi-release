---
id: REL-23
title: Filter sandbox publishing to transport inputs
status: To Do
assignee: []
created_date: '2026-10-23 17:33'
labels:
  - ci
milestone: m-0
dependencies: []
references:
  - >-
    https://github.com/Archont561/pixi-sandbox/blob/v0.8.0/crates/pixi-sandbox-core/src/sandbox_config.rs
priority: medium
type: chore
ordinal: 23000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The adopted publisher repacked ~400 MiB of branch history on every push to main, including pushes that only touched backlog Markdown or Rust sources, which cannot change the packed environment. pixi-sandbox 0.8.0 supports a [workflow] table whose push_paths and relock_paths become the two path allowlists; without it the derivation is the same set, but nothing was committed to express it. This task records the policy as configuration and mirrors it into both generated workflows.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 pixi-sandbox.toml carries a [workflow] table whose push_paths and relock_paths match pixi-sandbox derive_push_paths and derive_relock_paths for this repository.
- [ ] #2 publish-sandbox.yml limits on.push to the push_paths set and relock.yml limits on.pull_request to the relock_paths set, with the hand-application reason recorded beside each block.
- [ ] #3 A docs or backlog-only push to main does not start a publish run, verified from the run list rather than assumed.
- [ ] #4 A push that changes pixi.toml, pixi.lock or pixi-sandbox.toml still starts a publish run that goes green.
- [ ] #5 actionlint passes locally over the edited workflows.
<!-- AC:END -->
