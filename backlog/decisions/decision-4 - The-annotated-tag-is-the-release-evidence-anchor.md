---
id: decision-4
title: The annotated tag is the release evidence anchor
date: '2026-10-04 12:32'
status: proposed
---
## Context

The lifecycle splits across two workflow runs: `release-prepare.yml` cuts and tags, and
`release-publish.yml` — a separate run, triggered later by the tag, and possibly resumed days
later — builds, verifies and publishes. The immutable plan has to survive that gap.

An early draft uploaded `plan.json` as a workflow artifact of the *prepare* run and then had
`verify` claim to check "against the plan" with no plan in hand. It was silently recomputing,
so the "immutable plan" was an assertion, not a property. Artifacts also expire (7 days by
default), which breaks resume exactly when resume is most needed.

Options considered: (a) commit `.pixi-release/plan.json` — forbidden by `AGENTS.md`, which
lists `.pixi-release/` as never-committed; (b) cross-workflow artifact download — expiring and
awkward; (c) attach the plan to a draft release — good for the publish half, but it does not
exist at cut time and cannot anchor the tag itself.

A read of `pixi-sandbox`'s `crates/xtask/src/commit_release.rs` found that it already creates
**annotated** tags (`git.tag_annotated`) with an **empty message**.

## Decision

**The annotated tag message carries the release evidence anchor.** `cut` writes:

```text
my-tool v1.2.3

pixi-release-plan: sha256:ab12cd…
pixi-release-policy: sha256:9f2c1d…
pixi-release-version: 0.1.0
```

`verify`, in a later workflow run, re-derives the plan from the **tagged tree + policy +
history**, hashes it, and compares against the trailer. A mismatch — policy edited after the
cut, history rewritten, tag moved — is a refusal, not a warning.

The tag message is free, immutable, replicated with the repository, signable, and never
expires. It is strictly better than any artifact for this job.

## Consequences

- The plan gains a real immutability property instead of an asserted one, and `plan` becomes a
  provably pure function of (tagged tree, policy, history) — testable without a forge.
- It costs nothing and breaks nothing: annotated tags are already what the family creates.
- Resume works indefinitely, not for seven days.
- **`cut` must be deterministic in the same sense the renderer is.** If re-deriving the plan
  from the tagged tree can differ from the plan `prepare` applied, every publish refuses. This
  makes plan determinism a hard, tested requirement rather than a nicety, which is the point.
- The trailer format should leave room for a signature. The digest proves the plan did not
  change; it does not prove *who* cut it. `git tag -s` plus a recorded key fingerprint would,
  and the trailer is already the right place — likely post-v1, but the format must not preclude
  it.
- Draft releases are a *separate* identity problem and are not solved by this: a draft has no
  git tag at all (`doc-5`), so the draft must be addressed by release ID. The tag anchors the
  plan; the release ID anchors the in-flight publication.
