---
id: REL-24
title: Vendor the Cargo registry into the published pack
status: Done
assignee: []
created_date: '2026-10-23 18:34'
updated_date: '2026-10-23 18:51'
labels:
  - ci
  - bootstrap
milestone: m-0
dependencies: []
references:
  - >-
    https://github.com/Archont561/pixi-sandbox/blob/v0.8.0/backlog/docs/adr/ADR-6%20-%20Vendored%20cargo%20crates%20live%20under%20.pixi-sandbox-vendor.md
priority: medium
type: chore
ordinal: 24000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Restoring the pack gave pixi, cargo, rustc, rustfmt, actionlint and bun, but not clippy, check or test: crates.io is unreachable and the pack shipped no crates. With cargo_vendor = true the packer runs cargo vendor --locked into .pixi-sandbox/vendor/ (ADR-6) and restore wires a sandbox-owned CARGO_HOME, so cargo resolves offline from the pack. Gated on REL-21: a stub lock makes cargo vendor --locked abort and would turn every publish red.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 pixi-sandbox.toml sets cargo_vendor = true and the publish run packs .pixi-sandbox/vendor/ into the sandbox branch.
- [x] #2 The packed vendor tree is derived from the committed Cargo.lock, so two publishes of the same commit produce the same crate set.
- [x] #3 A pack restored on a machine with no crates.io access completes cargo check --offline or an equivalent build in the restored environment.
- [x] #4 Pack size and file count are measured and recorded, since vendoring grows the branch history.
<!-- AC:END -->

## Comments

<!-- COMMENTS:BEGIN -->
created: 2026-10-23 18:51
---
2026-10-23. Verified end to end. Publish run 40005070787 shows the matrix entry ending in ', true' and success; the branch snapshot message now reads '(default · linux-64 · vendored crates · schema 3)'. doctor on the cloned pack reports: vendor 209 crates, 272.2 MiB, Loose mode, Cargo.lock 01f65d0eb8c0; payload 747.7 MiB (envs 400.3, tools 75.2, vendor 272.2 = 36% vendor); 9072 blobs, 748.5 MiB verified with 0 failures. Clone size 1.3 GB, 9021 vendored files.

The payoff: restore wired cargo at .pixi-sandbox/cargo-home, and 'pixi run --frozen -- cargo check --workspace --all-targets --offline' compiled every crate from the vendor tree with crates.io unreachable. clippy -D warnings clean, 11 tests pass, and 'pixi run --frozen gates' exits 0 offline - the first time the full gate set has run in this sandbox.

One integration defect found and fixed here: turbo 2 defaults to strict env mode, so CARGO_HOME never reached cargo inside turbo run and lint tried the network. turbo.json now lists CARGO_HOME in globalPassThroughEnv.
---
<!-- COMMENTS:END -->
