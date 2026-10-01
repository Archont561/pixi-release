---
id: REL-1
title: Scaffold the reproducible Pixi–Bun–Turbo repository
status: Done
assignee: []
created_date: '2026-10-01 00:27'
updated_date: '2026-10-01 00:27'
labels:
  - bootstrap
  - ci
milestone: m-0
dependencies: []
documentation:
  - backlog/docs/architecture/doc-1 - Release-controller-architecture.md
priority: high
type: chore
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Create the independent `pixi-release` repository skeleton with a Linux Pixi lock, a Bun workspace lock, a Cargo workspace lock, dummy Rust crates, a Turbo facade for Cargo, a docs workspace package, canonical CI, and Lefthook hooks.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A fresh clone can run pixi install --locked and pixi run ci.
- [x] #2 crates/package.json and docs/package.json participate in the same Turbo graph.
- [x] #3 CI and Lefthook call pixi run tasks rather than direct cargo, bun, or turbo commands.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Committed locks exist for Pixi, Bun, and Cargo.
- [x] #2 pixi run lint, typecheck, test, build, lint-actions, and workflow-check pass.
<!-- DOD:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented and validated the scaffold locally through the canonical Pixi task graph.
<!-- SECTION:FINAL_SUMMARY:END -->
