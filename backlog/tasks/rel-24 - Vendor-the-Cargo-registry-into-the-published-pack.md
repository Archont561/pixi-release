---
id: REL-24
title: Vendor the Cargo registry into the published pack
status: To Do
assignee: []
created_date: '2026-10-23 18:34'
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
Restoring the pack gave pixi, cargo, rustc, rustfmt, actionlint and bun, but not clippy, check or test: crates.io is unreachable from the authoring sandbox and the pack shipped no crates. Setting cargo_vendor = true makes the packer run cargo vendor --locked into .pixi-sandbox/vendor/ (ADR-6), and restore wires a sandbox-owned CARGO_HOME so pixi run -- cargo build --offline resolves from the pack. This is gated on REL-21: with a stub Cargo.lock, cargo vendor --locked aborts with cannot update the lock file because --locked was passed, which would turn every publish red.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 pixi-sandbox.toml sets cargo_vendor = true and the publish run packs .pixi-sandbox/vendor/ into the sandbox branch.
- [ ] #2 The packed vendor tree is derived from the committed Cargo.lock, so two publishes of the same commit produce the same crate set.
- [ ] #3 A pack restored on a machine with no crates.io access completes cargo check --offline or an equivalent build in the restored environment.
- [ ] #4 Pack size and file count are measured and recorded, since vendoring grows the branch history.
<!-- AC:END -->
