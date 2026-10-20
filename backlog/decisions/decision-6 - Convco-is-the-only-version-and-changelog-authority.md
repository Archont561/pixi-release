---
id: decision-6
title: Convco is the only version and changelog authority
date: '2026-10-20 18:56'
status: accepted
---
## Context

`AGENTS.md` names Convco the sole SemVer and changelog authority. The release design (`doc-1`, `doc-3` §4) says Convco stays authoritative when `project.version-source = "convco"`. Changesets were considered as an alternative for repositories that already use them, but adding a second authority contradicts that rule.

## Decision

Version bumps and changelog content come from Convco and conventional commit history, and from nothing else. `pixi-release` writes the version literal that Convco derives and stages the `CHANGELOG.md` that Convco generates. It does not author release notes and does not read `.changeset/`.

## Consequences

- Repositories that use changesets are out of scope until a separate decision adds a second authority and amends `AGENTS.md`.
- The `.versionrc` in this repository is the configuration point for the changelog.
- Release notes shown elsewhere (for example in a GitHub Release body) are rendered from the Convco changelog, not written by hand.
