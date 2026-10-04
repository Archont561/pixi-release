---
id: decision-1
title: Use Pixi → Bun → Turbo as the only quality-gate entrypoint
date: '2026-10-01 00:27'
status: accepted
---
## Context

Every current `Archont561` repository routes work through Pixi, and three of them
(`geoquery`, `pathway`, `qgis-rs`) fan out through a root Bun workspace into Turbo with a
`crates/package.json` façade making Cargo a Turbo node. `pixi-sandbox` deliberately does not:
its `decision-3` defers Turbo until measurable triggers are met — ≥3 JS packages, a real
cross-package import edge, and ≥60 s of repeated JS CI work (or >10 s local feedback for
unchanged packages).

By those triggers `pixi-release` has not earned Turbo. It has two workspace members (`crates`,
`docs`), no cross-package edge, and a near-zero JS build. Adopting Turbo here is therefore a
deliberate exception that needs a reason specific to this product, not an inherited habit.

## Decision

**Pixi → Bun → Turbo is the single route, and the reason is the product, not the repository.**

```text
pixi run <verb> → root package script → bun x turbo run <verb> → package script
```

`pixi-release` must render correct workflows for Turbo-shaped consumers (`geoquery`, `pathway`,
`qgis-rs`) *and* non-Turbo ones (`pixi-sandbox`, `nadir`). A renderer that hardcodes
`bun x turbo run test` is wrong for two of five peers; a renderer whose authors have never run
the Turbo path is wrong in subtler ways. Dogfooding the more complex of the two shapes is the
cheaper mistake.

Two obligations follow from accepting the exception:

1. **The renderer must never assume Turbo.** `.pixi-release.toml` names verbs and targets; how
   a repository implements `pixi run test` is that repository's business. The policy file does
   not contain the words `cargo`, `turbo`, `bun` or `pytest` (see decision-2).
2. **`turbo.json` stays minimal.** Every Cargo-backed node is `cache: false`, as
   `crates/turbo.json` already declares — Cargo owns that cache. Caching cleverness the product
   cannot justify does not belong here.

Never add a second CI or hook command path. `lefthook.yml` and `.github/workflows/*` call only
`pixi run <verb>`, with the single documented exception in decision-2 §Consequences (generated
consumer files call the controller binary directly, because the controller is not a dependency
of the consumer's project).

## Consequences

- One command surface: `pixi run gates` locally is the CI gate, and a reviewer reads one graph.
- The exception is written down and bounded. If the renderer is ever found branching on
  "is this a Turbo repo?", obligation 1 has been broken and this decision is the thing to cite.
- If `pixi-release` permanently remains two workspace members with no cross-package edge, the
  honest follow-up is to re-examine whether the dogfooding argument still outweighs the
  dependency — the same re-check `pixi-sandbox`'s decision-3 demands when its triggers move.
- Scope Turbo work by forwarding an explicit filter, e.g.
  `pixi run test -- --filter=@pixi-release/rust`.
