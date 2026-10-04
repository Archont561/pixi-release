---
id: decision-2
title: 'pixi-release is a release controller, not a build system'
date: '2026-10-04 12:32'
status: proposed
---
## Context

An early draft of the consumer release workflow rendered `pixi-release build --target <triple>`
into the build matrix. That quietly made the controller a build system: it would have to know
what a target triple means, how to drive five toolchains, and how to strip a Mach-O binary.
`doc-1` says the opposite — *"It is not a generic CI generator"* — and the survey
(`doc-4`) shows the consumer population is not even uniformly Rust: `geoaiworkbench` publishes
Python wheels and has no Cargo workspace at all.

The competing option was a renderer that grows a code path per artifact kind (Rust binary,
wheel, npm tarball, conda package). That is unbounded work and it makes every new consumer
shape a release of `pixi-release`.

## Decision

> `pixi-release` decides **what** must be built, **where**, and **what must come back**.
> The repository decides **how**. The controller never runs a compiler.

The build matrix is therefore **policy data, not tool knowledge**. `.pixi-release.toml` carries
`[build].task`, `[build].environment` and a list of `[[build.matrix]]` entries, each naming
`platform`, `runner`, `args` and the `artifacts` globs it must produce. The renderer copies
those values into the workflow verbatim.

In the rendered build matrix, every step is the repository's own command except one: `collect`,
which resolves the declared globs, refuses an empty match, names canonically, and hashes. It
compiles nothing. The rendered form is `doc-2` §3.2.

This yields the two-part invocation rule:

**Controller work is `pixi-release <verb>`. Repository work is `pixi run <task>`.**

## Consequences

- A wheels-only repository is supported by **configuration** — one matrix entry and
  `artifacts = ["dist/*.whl", "dist/*.tar.gz"]` — with no new renderer code path.
- The win-64 problem becomes the consumer's, correctly: `environment = "package"` renders
  `run-install: false`, which is `pixi-sandbox`'s empty-environment trick expressed as a policy
  value instead of being hardcoded for everyone.
- What remains controller work is exactly what `doc-1` assigns it: immutable plans, artifact
  manifests, checksums, per-target evidence, and resume.
- **The documented exception to decision-1's one-command-surface rule.** Generated consumer
  workflows call `pixi-release <verb>` directly rather than `pixi run <task>`, because
  `pixi-release` is not a dependency of the consumer's project — it is a release-time tool
  installed globally at the stamped version. A rendered `pixi run release-cut` would depend on
  a task the consumer can edit, silently changing CI while the file still claims to be
  generated. `pixi-sandbox` already does this (`$(pixi-sandbox plan --config … --json)` in its
  generated publisher). The verb list is `doc-2` §4.
- The controller cannot diagnose a build failure beyond "your task exited non-zero", which is
  the right trade: it also cannot break a build it does not understand.
