---
id: REL-21
title: Commit a resolved Cargo.lock so the Rust gates are reproducible
status: To Do
assignee: []
created_date: '2026-10-23 15:57'
labels:
  - ci
  - bootstrap
milestone: m-0
dependencies: []
references:
  - 'https://github.com/Archont561/pixi-release/pull/12'
priority: high
type: chore
ordinal: 21000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Cargo.lock in the repository lists the four workspace members and no dependencies, so it is a stub rather than a resolved lockfile. Every cargo invocation therefore re-resolves the dependency graph: the diagnostic run for PR #12 reported "Locking 209 packages to highest Rust 1.98 compatible versions" before it could lint anything. Gate results are consequently not pinned by a reviewed artefact, which contradicts the AGENTS.md rule that lockfiles are committed and reviewed, and it means a dependency publication can change a green gate with no repository change. The resolved lock is a large reviewable file, which is why it was not folded into the lint fix.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A resolved Cargo.lock is committed and lists every transitive dependency with its resolved version and checksum.
- [ ] #2 The gates build with a lockfile-enforcing invocation (cargo --locked) so a dependency change cannot silently alter a run.
- [ ] #3 A fresh-clone ci run on main stays green after the lock lands.
- [ ] #4 The resulting diff is reviewed as a lockfile change rather than a code change (AGENTS.md, What not to commit).
<!-- AC:END -->
