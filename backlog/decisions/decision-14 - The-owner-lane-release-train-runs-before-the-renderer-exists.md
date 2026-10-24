---
id: decision-14
title: The owner-lane release train runs before the renderer exists
date: '2026-10-24 15:57'
status: accepted
---
## Context

`doc-3` §2.2 names `auto-release.yml` and `release.yml` as **owner → product** targets, and `doc-2`
§1.1 already tells a user to `pixi global install --channel https://prefix.dev/archont561/archont561
--channel conda-forge pixi-release`. Nothing produced any of it: the repository had `ci.yml` plus
the two files `pixi-sandbox` generates, and no way to turn a merge into a published artifact.

The renderer that will eventually emit these files for consumers is REL-5, and the dogfooding step
that pins a released controller is REL-7. Both come after the first release they are meant to
produce. This decision is about that order.

## Decision

1. **Both files are owner-lane now.** `.github/workflows/autorelease.yml` and
   `.github/workflows/release.yml` are hand-authored and marked *"Owner lane — not rendered"*
   (`doc-3` §2.1). Their drift gate is `repo-check` (immutable action SHAs, version agreement),
   not `pixi release workflow check`, which stays REL-12's job for rendered output.
2. **Convco stays the only version authority.** `autorelease.yml` derives the version with
   `convco version --bump` (`decision-6`); an explicit version is a human override of the same
   authority, not a second one.
3. **The tag is a bootstrap, not `cut`'s tag.** `autorelease.yml` writes the annotated tag with
   `git`, because `AGENTS.md` forbids controlling a release with the unreleased checkout and this
   repository has no `.pixi-release.toml` yet. Until REL-7 lands, the tag body is Convco's
   changelog section rather than `cut`'s `pixi-release-version` / `plan-digest` / `policy-digest`
   trailers. **Deletion trigger:** REL-7 replaces this step with a pinned, verified prior
   `pixi-release` running `plan → prepare → cut`, and `decision-4`'s trailers become the tag body.
4. **Milestone tags are dispatch triggers, not evidence.** `milestone-v*`, `milestone/*` and `m-v*`
   ask the train for a release. They carry no version, anchor no evidence, and every release still
   produces exactly one `v*` tag — so `decision-4` (single annotated anchor) and `decision-7` (one
   version, one tag) stand unchanged. REL-16 keeps ownership of scoped and per-package tags.
5. **A tag pushed with `GITHUB_TOKEN` starts no workflow** (`doc-3` §2.3 rule 1), so
   `autorelease.yml` dispatches `release.yml` explicitly with `gh workflow run`. That step needs
   `actions: write`; deleting either the step or the permission stops every release silently.
6. **Publication is OIDC-only.** The artifact goes to prefix.dev channel `archont561/archont561`
   from a job running under the `prefix` environment with `id-token: write`. The workflow file name
   and the environment name are part of the trusted-publishing binding: renaming either breaks
   publishing. No registry token is stored in the repository or in Actions secrets.
7. **A prerelease never spends registry version space.** A `vX.Y.Z-suffix` tag builds, checksums
   and publishes a GitHub prerelease, and skips the channel upload.
8. **Platform scope is `linux-64` until a matrix exists.** `pixi.toml` locks one platform and the
   recipe builds a compiled artifact, so a release publishes one platform. A multi-platform matrix
   is separate work, tracked as a gap rather than implied by these files.

9. **A merge never spends a version.** `autorelease.yml` runs on three events and treats them
   differently: a `main` push **observes** (it computes the version Convco would give and stops),
   while a milestone tag or a `workflow_dispatch` **asks** (gates → packaging proof → notes → tag →
   dispatch `release.yml`). The first version of this train tagged on any qualifying merge, which
   made versions an accident of how often `main` moved and made the milestone tag redundant. The
   cost is deliberate: the packaging proof runs in the release run rather than on every merge, so
   `ci.yml` stays the answer to *is `main` healthy* and the release run is the answer to *does the
   artifact build*. A dispatch off a branch is still a release request — the event, not the ref,
   decides.

## Consequences

- `REL-27` tracks the train. Its end-to-end proof is one dispatched run, because YAML that
  actionlint accepts has not thereby ever executed. The first such run (40109716135) proved the
  point by failing: a `main` push reached the packaging step and stopped there, so §9 exists
  because that run should never have been cutting a tag at all.
- `REL-7` inherits step 3's bootstrap and is expected to delete it.
- The consumer-facing shapes REL-5 renders are *not* these files. When REL-5 lands, the two must be
  compared deliberately: a bootstrap that diverges from the rendered target is how a generator bug
  hides (`doc-3` §2.1).
