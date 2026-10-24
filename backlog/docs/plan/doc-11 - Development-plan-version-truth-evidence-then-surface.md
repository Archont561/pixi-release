---
id: doc-11
title: Development plan - version truth, evidence, then surface
type: guide
created_date: '2026-10-24 18:12'
updated_date: '2026-10-24 18:12'
---
## Status

Written 2026-10-24, after `v0.1.0` shipped and the owner lane ran end to end (REL-27). This document
decides the **order** of the remaining work and the two decisions that order needs. It supersedes the
sequencing in `doc-10` §5; it does not supersede `doc-10`'s sprint scope, which stays authoritative for
Sprint 1. Where this document and a task disagree, the task is the work and this document is the order.

A companion survey of the two sibling repositories is not repeated here: the evidence is in the
decisions and tasks this document orders, each cited where it is used.

## 1. The three ordering constraints

Every sequence below is forced by one of these. Nothing else justifies an order.

1. **The schema must change before it is rendered.** `version.manifests` is a flat list of paths and
   `prepare` guesses each file's format. Every item on the capability list — many languages, many
   modes, consistency, updating, per-version releases — reads that list. Change it once, in `REL-11`
   under `decision-16`, before `REL-5` writes it into consumers' repositories.
2. **Nothing is resumable until evidence exists.** Resume, scoped release, per-artifact version and
   prerelease-without-push all consume the same record of what was produced, verified and published.
   That is `REL-6`, and it is the largest single win: the `v0.1.0` failure cost a hand-built workflow
   branch because no such record existed.
3. **A publish job must consume a tree that already agrees with its tag.** Today `release.yml` stamps
   the version after checkout of an already-pushed tag and never commits it, so the artifact is built
   from a tree the tag does not describe. That is why `REL-27` AC#2 cannot be proved today, and
   `REL-30` is what makes it provable.

## 2. The waves

| Wave | Capability | Work | Milestone |
|---|---|---|---|
| 0 | version updating, ordering | `REL-29` prepare owns the write set — internal pins, lockfiles, generated files; `REL-30` prepare → commit stages exactly the touched set → tag last, and `release.yml` loses its stamp step | m-1 |
| 1 | many languages, consistency | `REL-11` schema 2 (`decision-16`); then `REL-28` one checker over declared formats plus `verify`; then `REL-31` documented references, stamped and derived | m-2 |
| 2 | resumable workflows | `REL-6` evidence and per-target resume; then `REL-32` retires the `existing_run_id` input | m-2 |
| 2 | fast workflows | `REL-33` reuse the gate evidence already recorded for the commit instead of re-running it | m-2 |
| 3 | many modes, fast | `REL-5` renderer, `REL-12` drift gate, `REL-7` dogfood | m-1, m-2 |
| 4 | per-version, scoped | `REL-16` under `decision-17`; scoped planning over Convco | m-3 |
| 5 | registries, prerelease without pushing | `REL-13`, `REL-14`, `REL-15` | m-3 |

Waves 0 and 1 are the whole of the near-term product, and wave 0 is ordered first on purpose: the
write set and the tag order need no schema change, and `REL-30` is what lets `REL-27` AC#2 be proved
at the next cut. Wave 3–5 are already in the backlog and are ordered here only to say what must not
start first.

## 3. Modes, as one flag set

A mode is an input to `plan`, honoured by every verb, not a separate code path.

| mode | meaning |
|---|---|
| `--dry-run` | plan and prepare, report the diff, change nothing, push nothing |
| `--no-push` | build, verify and tag locally; nothing leaves the machine |
| `prerelease` | build, verify, record evidence; upload to no registry |
| `--resume` | consume existing evidence; repeat no target that already succeeded |
| `--scope <artifact>` | plan one artifact of a release group (`decision-17`) |

`prerelease` today skips only the registry upload while still creating a GitHub Release. That is the
gap `--no-push` closes, and it belongs to `REL-6`, where the evidence model decides what "published"
means per target.

## 4. Fast means measured

The runs on record: `ci` gates 2m17–2m48, `autorelease` 2m10 (gates plus packaging), `release` build
50s + registry 31s + GitHub Release 8s.

The largest waste is not caching. A release re-executes the entire gate suite at a commit where `ci`
already passed. `REL-33` reads the gate evidence recorded for that exact SHA and refuses when it is
absent — the same discipline `decision-8` applies to binaries, applied to proof. Caching and matrix
parallelism come after that, and only against a measured baseline recorded in the task.

## 5. What is deliberately not started

- **`REL-19` / `REL-20`, the documentation site.** Post-v1, adds a deployment surface, and depends on
  the manifest schema that has not changed yet.
- **Registry targets** before `REL-6` and `REL-13`. `decision-9` already forbids them, and each registry
  has a different auth story: OIDC for npm, PyPI and nuget.org, a Portal token plus a GPG key for Maven
  Central, OIDC for prefix.dev. That difference is policy, and policy comes after evidence.
- **A second release implementation in another language.** Three repositories now hold three copies of
  these ideas (`scripts/release.py`, `crates/pixi-release/src/release/`, and this workspace). The
  duplication is the problem this product exists to remove, not a model to copy.

## 6. Decision state

- `decision-16` is **accepted** (2026-10-24), with detection and the environment source added to it:
  a closed table resolves format, key and scheme for a bare path, an unrecognised file is refused
  rather than guessed at, and one policy-named environment variable may only *agree* with a version,
  never supply one.
- `decision-17` stays **proposed** and dormant. It gates `REL-16`, which is m-3, and nothing in this
  plan depends on it. A proposal is not a licence to build against it.
- `decision-3` and `decision-5` remain **proposed**, which is why `REL-5` and `REL-12` are not in the
  near-term waves: rendering generated workflows under a proposal would decide it by accident.

## 7. What "done" means for this plan

Each wave ends with a green `gates` run on its merge commit and its acceptance criteria checked from
run evidence, never from intent. A wave that cannot show a run stays open, with the missing proof
named in its task — the discipline `REL-27` already follows.
