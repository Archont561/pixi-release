---
id: decision-10
title: 'Consumer entrypoint, renderer scope, pixi-sandbox ownership and seeding'
date: '2026-10-20 19:51'
status: accepted
---
## Context

`doc-3` §6 left four questions open and blocked M1 sizing (REL-9). `pixi-sandbox` at `main` has no
`setup/action.yml`, so the `uses:` pins in `geoquery` and `qgis-rs` point at nothing.

## Decision

1. **Consumer entrypoint.** Consumers install the release tooling with `pixi global install` from the
   project channel `https://prefix.dev/channels/@archont561/archont561`. Rendered workflows contain a
   bootstrap step, not a `uses:` line. No composite action is shipped in v1.
2. **Renderer scope.** `pixi-release` v1 claims only the release lane. `ci.yml` and `docs.yml` stay
   hand-written in each consumer.
3. **pixi-sandbox relationship.** `pixi-sandbox` owns `publish-sandbox.yml` and `relock.yml`.
   `pixi-release` owns `release.yml` and `auto-release.yml`. Each tool uses distinct markers and
   stamps, and each has its own drift bot.
4. **Seeding.** `pixi-release` v0.1.0 is a clean-room implementation against `doc-1`. It does not
   copy the `pixi-sandbox` `xtask` code.

## Consequences

- REL-5 renders the bootstrap step and only the release-lane workflows.
- REL-12 uses `pixi-release`'s own markers and stamps and does not reuse `pixi-sandbox`'s.
- REL-9 is answered. `doc-3` §6 and `doc-9` record the answers.
- The clean-room choice means REL-2 to REL-4 are implemented from `doc-1`. Tests from `pixi-sandbox`
  may be used as reference but not copied wholesale.
