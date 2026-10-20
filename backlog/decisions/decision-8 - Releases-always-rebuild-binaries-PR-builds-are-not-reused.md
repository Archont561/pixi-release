---
id: decision-8
title: Releases always rebuild binaries; PR builds are not reused
date: '2026-10-20 18:56'
status: accepted
---
## Context

`cut` writes version literals and commits them, so the tagged commit differs from the commit a PR built. A binary that embeds its version (for example through `--version`) would carry the wrong version if built before `cut`. Reuse would also depend on GitHub Actions artifacts, which expire (`decision-4`), and on a tree-equality check that is not yet specified.

## Decision

A release builds its binaries after `cut`, from the tagged commit, in the publish workflow. No binary built on a pull request or an earlier run is reused for a release.

## Consequences

- Release time is the build time. Triage still skips the build matrix when it is not needed (`doc-2` §3.2), so resumed publishes do not rebuild.
- Speed-ups must come from caching or triage, not from reusing artifacts. Revisit this only after the rebuild cost is measured.
