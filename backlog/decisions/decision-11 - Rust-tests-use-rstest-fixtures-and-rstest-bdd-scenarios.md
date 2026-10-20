---
id: decision-11
title: Rust tests use rstest fixtures, proptest properties and rstest-bdd scenarios
date: '2026-10-20 20:30'
status: accepted
---
## Context

The scaffold tests used plain `#[test]` functions and built no shared test data. Dummy release
repositories were going to be built inline in each test.

## Decision

1. Rust tests use `rstest` fixtures and `#[rstest]` cases. Plain `#[test]` is not used.
2. Invariants that must hold for all inputs (determinism, ordering, idempotence, transition
   rules) are `proptest` properties inside an `#[rstest]` test. Each property names its seam.
   Failing seeds are committed to `proptest-regressions/` as part of the fix.
3. Tests follow the `tdd` skill: test first at a seam agreed beforehand, red before green, one
   vertical slice at a time. Refactoring happens in review, not inside the loop.
4. Behaviour that a reader should follow in plain language is written as `rstest-bdd` scenarios
   in a `tests/features/*.feature` file, bound with `#[scenario]`.
5. Dummy repositories are built by one plain builder function per crate, in a `tests/support/`
   module. Tests receive them through an rstest fixture that wraps the builder, so every fixture
   case gets a fresh repository. Tests never build repositories inline.
6. Versions are pinned once in the workspace: `rstest` 0.26.1 with `rstest-bdd` and
   `rstest-bdd-macros` 0.6.0, the pairing the rstest-bdd README documents, and `proptest` 1.12.0.

## Consequences

- Every implementation task with tests (REL-2 to REL-8, REL-11, REL-12) carries an acceptance
  criterion that requires these conventions.
- The first `cargo test` run is the first compile check of these dependencies. Cargo is not
  installed in the scaffold sandbox, so the conventions are not yet verified by a build.
