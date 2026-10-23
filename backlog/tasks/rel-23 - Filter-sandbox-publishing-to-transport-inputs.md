---
id: REL-23
title: Filter sandbox publishing to transport inputs
status: Done
assignee: []
created_date: '2026-10-23 17:33'
updated_date: '2026-10-23 17:48'
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
The adopted publisher repacked ~400 MiB of branch history on every push to main, including pushes that only touched backlog Markdown or Rust sources, which cannot change the packed environment. pixi-sandbox 0.8.0 supports a [workflow] table whose push_paths and relock_paths become the two path allowlists. This task commits the policy as configuration and mirrors it into both generated workflows; the mirror is hand-applied because init needs conda-forge, which this sandbox cannot reach.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 pixi-sandbox.toml carries a [workflow] table whose push_paths and relock_paths match pixi-sandbox derive_push_paths and derive_relock_paths for this repository.
- [x] #2 publish-sandbox.yml limits on.push to the push_paths set and relock.yml limits on.pull_request to the relock_paths set, with the hand-application reason recorded beside each block.
- [x] #3 A docs or backlog-only push to main does not start a publish run, verified from the run list rather than assumed.
- [x] #4 A push that changes pixi.toml, pixi.lock or pixi-sandbox.toml still starts a publish run that goes green.
- [x] #5 actionlint passes locally over the edited workflows.
<!-- AC:END -->

## Comments

<!-- COMMENTS:BEGIN -->
created: 2026-10-23 17:42
---
2026-10-23. Local verification with the environment restored from the published pack (pixi install --frozen --offline, cargo 1.98.1): actionlint exit 0 over all three workflows; pixi-sandbox.toml parses; a comparison script reports the two YAML lists identical to the config lists. CI green on PR #17.

Behavioural proof in two halves. Positive: PR #17 changed pixi-sandbox.toml, so the merge to main (4639d7f) started publish sandbox (run 39996560021) and it went green, republishing sandbox/developer-linux-64 at eac9498 — the filter does not block a real transport change. Negative: the commit that closes this task touches only backlog/, and no publish sandbox run exists for it. Both checked from the run list, not assumed.
---

created: 2026-10-23 17:48
---
Negative probe confirmed after merge: backlog-only commit 8f5bb40 produced only a ci run (39997940859) and no publish sandbox run at all; sandbox/developer-linux-64 stayed at eac9498 rather than repacking. The relock filter showed the same behaviour on PR #18 - the workflow did not appear in the checks because its pull_request.paths excluded a backlog-only change.
---
<!-- COMMENTS:END -->
