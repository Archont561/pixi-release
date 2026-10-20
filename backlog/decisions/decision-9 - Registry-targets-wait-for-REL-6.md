---
id: decision-9
title: Registry targets wait for REL-6
date: '2026-10-20 18:56'
status: accepted
---
## Context

`AGENTS.md` forbids adding a registry target until its artifact manifest and resume behaviour are specified and tested. REL-6 is the task that does that work. The sidecar component discussed for release presentation (`doc-8`) would be published to npm, which is a registry target.

## Decision

Only `github-release` is an admissible `[[target]]` before REL-6 is done. npm, PyPI, prefix.dev and any other registry target are added only after REL-6 specifies and tests their artifact manifest and resume behaviour.

## Consequences

- The release presentation component is a consumer repository. Its own npm release waits for REL-6.
- No install command for a registry appears in generated output until that registry is an admitted target.
