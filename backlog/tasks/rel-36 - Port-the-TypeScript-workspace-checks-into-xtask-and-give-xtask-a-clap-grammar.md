---
id: REL-36
title: Port the TypeScript workspace checks into xtask and give xtask a clap grammar
status: In Progress
assignee: []
created_date: '2026-10-24 19:38'
updated_date: '2026-10-24 19:38'
labels:
  - chore
milestone: m-1
dependencies: []
priority: medium
ordinal: 36000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
crates/xtask/scripts/ carried the only TypeScript in the repository: verify-workspace.ts (required files, root workspace declaration, Cargo membership) and workspace.test.ts (three assertions about package.json, bun.lock and the absent npm facade). Answering a question about Cargo.toml therefore needed a Bun install, a Turbo task and a coverage path, and the root package.json had four scripts whose entire job was to run them. Port every assertion to Rust in xtask, delete the scripts and their wiring, and move xtask's hand-rolled argv parsing onto clap so the repository has one CLI dialect (decision-13). Bun and Turbo remain the gate entrypoint (decision-1); only the TypeScript leaves.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 crates/xtask/scripts/ is gone, and no workflow, pixi task, package script, turbo task or rust source still refers to it.
- [x] #2 Every assertion the TypeScript made is a Rust check or test: required files, root workspaces declaration, crates/xtask as a Cargo member, no crates/xtask/package.json, bun.lock agreeing with the workspace list, and no workspace:docs.
- [x] #3 xtask parses argv with clap, keeps check --tag, exits 2 for a usage error, and has a test proving the grammar refuses a verb it does not ship.
- [x] #4 pixi run gates stays green, including bun install --frozen-lockfile, and the Cargo.lock records xtask's two dependencies.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 The check fails loudly when a second npm facade reappears under crates/xtask (verified by creating one).
- [x] #2 README and pixi task descriptions no longer describe a TypeScript suite.
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented on the session branch. Where each TypeScript assertion landed: required files → `required_files` (now also `crates/xtask/Cargo.toml` and `bun.lock`); root workspaces + Cargo membership + no facade → `workspace_contract`, a repository check, so it runs in `pixi run repo-check` on every PR instead of only inside `bun test`; the bun.lock layout and the 'no workspace:docs' invariant → `the_bun_lockfile_agrees_with_the_workspace_layout`. Grammar: `clap` derive on `Cli`/`Command::Check { tag }`, with `clap_owns_the_check_grammar` proving the verb list, the optional `--tag`, and that a `release` verb does not exist.

Measured: `pixi run --frozen gates` exit 0 with the scripts deleted — bun install --frozen-lockfile reports no changes, clippy/fmt clean, all suites pass, `xtask check: repository checks passed`. `xtask --help` lists only `check`; no verb exits 2; `check --tag v0.1.0` exits 0; `check --tag v9.9.9` exits 1 with four findings. Negative case for the ported contract: creating `crates/xtask/package.json` makes `xtask check` exit 1 naming the facade; removing it returns to 0. Cargo.lock gained xtask → { clap, tempfile }, both already resolved in the workspace.
<!-- SECTION:NOTES:END -->
