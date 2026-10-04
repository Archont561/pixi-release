---
id: decision-3
title: >-
  Generated workflows are marked, stamped, drift-gated, and never float a
  production pin
date: '2026-10-04 12:32'
status: proposed
---
## Context

`pixi-sandbox` already renders `relock.yml` and `publish-sandbox.yml` into five consumer
repositories, and the survey (`doc-4`) measured the result: `relock.yml` is byte-identical
across `geoquery`, `pathway` and `pixi-sandbox`, and every rendered file opens with an
ownership marker and a `# pixi-sandbox-version:` stamp. The counter-example is in the same
account — `nadir/.github/workflows/publish-sandbox.yml` is an **unstamped, stale 59-line copy**
of a 292-line generated file. That is what the absence of this contract looks like in the wild.

`pixi-sandbox`'s `decision-4` ("Consumer binaries self-update only to propose reviewed
regenerated files, and production pins never float") solved this for its own two files. The
question is whether `pixi-release` re-derives a policy or adopts that one.

The alternatives were (a) float the workflow to `@v0`/latest so new releases are picked up
implicitly, and (b) keep upgrades manual and docs-only.

## Decision

**Adopt `pixi-sandbox`'s decision-4 substantially unchanged, with one addition.**

1. **Marker + stamp.** Every rendered file opens with an ownership marker and
   `# pixi-release-version: X.Y.Z`. The stamp *is* the pin: the same version appears in the
   workflow's own `env:` block and is what the one-line
   `pixi global install … "pixi-release==${PIXI_RELEASE_VERSION}"` resolves. One version
   domain, not two.
2. **A policy digest — the addition.** Each rendered file also carries
   `# pixi-release-policy: sha256:…`, the digest of `.pixi-release.toml` at render time. The
   `pixi-sandbox` stamp leaves a hole: editing the config without re-rendering silently
   desynchronises policy from the workflows that enforce it. The digest closes it.
3. **`pixi release workflow check` is the drift gate.** Render fresh into a temp directory,
   compare files and policy digest, exit non-zero, **write nothing**.
4. **Upgrades arrive as a reviewed pull request.** `release-upgrade.yml` runs on a schedule,
   bootstraps the *exact* pinned version, asks that verified binary to self-update, re-renders
   on drift, and opens a PR. The unit of upgrade is "self-update, then re-render" — never a
   hand-edited version constant, because the template changes with the version.
5. **Config is never rewritten.** `init` refuses to overwrite an existing `.pixi-release.toml`.
   An older `schema` is reported as a finding naming its migration path. *The gate is the
   review, not the edit.*
6. **Production pins never float.** `latest` is permitted only inside the upgrade job, only to
   propose a PR. Every committed workflow names an exact version; rollback selects an exact
   older one.

Floating pins (option a) are rejected for the reasons `pixi-sandbox` gives, and silence
(option b) is rejected because `nadir` is the measured outcome of it.

## Consequences

- Consumers get release-drift detection without surrendering review: one PR per release worth
  of template changes; config changes are never automated.
- `workflow check` must be deterministic — render twice, compare — or the drift gate becomes a
  source of spurious PRs. `doc-5`'s determinism probe is the precedent and the proof obligation.
- Every rendered step must stay a pinned `uses:` or a single `pixi-release`/`pixi run` line.
  `pixi-sandbox`'s own upgrade job breaks this three times and its `consumer-proof.yml` four
  more, each marked `# multiline-run-allowed`; the fix is to ship `self update`,
  `workflow check`, `upgrade pr` and `proof run` as subcommands so each is one line.
- A repository adopting both tools gets two scheduled drift jobs, two upgrade PRs and two
  marker conventions. Whether the two tools share a marker/stamp format is a cross-project
  question this decision deliberately leaves open.
