---
id: REL-21
title: Commit a resolved Cargo.lock so the Rust gates are reproducible
status: Done
assignee: []
created_date: '2026-10-23 15:57'
updated_date: '2026-10-23 18:51'
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
The committed Cargo.lock was a stub listing only the four workspace members, so every cargo invocation re-resolved 209 packages and the Rust gates were not pinned by a reviewed artefact. Fixed: a resolved lock (213 packages, 1987 lines) is committed, and the gates pass --locked to cargo build, test, check and clippy.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A resolved Cargo.lock is committed and lists every transitive dependency with its resolved version and checksum.
- [x] #2 The gates build with a lockfile-enforcing invocation (cargo --locked) so a dependency change cannot silently alter a run.
- [x] #3 A fresh-clone ci run on main stays green after the lock lands.
- [x] #4 The resulting diff is reviewed as a lockfile change rather than a code change (AGENTS.md, What not to commit).
<!-- AC:END -->

## Comments

<!-- COMMENTS:BEGIN -->
created: 2026-10-23 18:34
---
2026-10-23. A resolved Cargo.lock is committed (213 packages, 1987 lines, proptest 1.12.0 pinned among them), generated on a runner because crates.io is unreachable from the authoring sandbox. The gates now pass --locked to cargo build, test, check and clippy, so a dependency change can no longer silently alter a run. AC#3 (a fresh-clone ci run stays green) is proven by the gates run on this change; the vendoring follow-up is REL-24.
---

created: 2026-10-23 18:51
---
Done 2026-10-23. Lock generated on a runner (crates.io is unreachable from the authoring sandbox) and committed as e90c351. cargo fmt, clippy -D warnings and the full test suite (11 tests) now run offline against it, with gates exit 0.
---
<!-- COMMENTS:END -->
