---
id: doc-9
title: Roadmap
type: specification
created_date: '2026-10-20 19:06'
updated_date: '2026-10-23 15:01'
---

## Status

Adopted 2026-10-20. The task and milestone changes below are applied in Backlog. The versions are
expectations. Convco computes the real version from commit history (`decision-6`).

Known limit: the Backlog CLI cannot edit a milestone description. The descriptions of m-0, m-1 and
m-2 therefore still show the older outline. This doc and the task files are authoritative where they
differ. m-3 was added with the current description.

## Version mapping

| Version | Milestone | Exit condition | Tasks |
|---|---|---|---|
| v0.1.0 | M0 | `pixi run ci` green from a fresh clone, verified on `main` | REL-1 (done), REL-10 |
| v0.2.0 | M1 | A dry run proves why a tag may or may not be cut, without touching Git state | REL-2, REL-3, REL-4, REL-5, REL-8, REL-9 |
| v0.3.0 | M2 (publish part) | Every required target has deterministic evidence and a recoverable publication path | REL-6, REL-11, REL-12 |
| v1.0.0 | M2 (self-host part) | The N−1 self-release is verified and the consumer proof passes | REL-7 |

Only `github-release` is an admissible target before REL-6 is done (`decision-9`). The first
tagged release that exercises a registry target comes after that.

## Task mapping

| Task | Milestone | Depends on | Scope |
|---|---|---|---|
| REL-1 | M0 | — | Scaffold the reproducible Pixi–Bun–Turbo repository (done) |
| REL-10 | M0 | REL-1 | Prove M0 with a green run of `pixi run ci` on `main` |
| REL-2 | M1 | REL-1 | Parse and validate the authored release policy |
| REL-3 | M1 | REL-2 | Produce immutable deterministic release plans |
| REL-4 | M1 | REL-3 | Prepare and cut behaviour, with the touched-set refusal |
| REL-8 | M1 | REL-2 | Generate the changelog through Convco (`decision-6`) |
| REL-9 | M1 | — | Answer the open questions in `doc-3` §6 |
| REL-5 | M1 | REL-3, REL-9 | Renderer only: templates, golden fixtures, the three consumer shapes |
| REL-11 | M2 | REL-4 | Define the release manifest schema (`doc-8` base) |
| REL-12 | M2 | REL-5 | Drift gate: markers, stamps, `workflow check`, PR bot |
| REL-6 | M2 | REL-4, REL-11 | Evidence JSON, per-target resume, five-platform matrix, OIDC publish |
| REL-7 | M2 | REL-5, REL-6 | Dogfood sandbox publication and verified N−1 self-release |

Each task that has tests gained the criterion "Proven by a green gates run on the merge commit
that covers this task's tests". REL-7 also requires a verified N−1 self-release.

## Revisions to the existing milestones

- **M0.** Scope unchanged. The exit condition is not yet demonstrated, because CI has not completed
  green on this branch. M0 closes with REL-10, not with REL-1 alone.
- **M1.** Renderer work (REL-5) and changelog generation through Convco (REL-8) are included. The
  open questions (REL-9) must be answered first.
- **M2.** Evidence and publication (REL-6), the manifest schema (REL-11) and the drift gate
  (REL-12). REL-7 stays last.
- **REL-5 placement.** The renderer (M1) and the drift gate (M2) are split into REL-5 and REL-12.
  Each now sits in its own milestone.

## Scope outside the milestones

These are real work, deliberately not in v1:

- **Presentation sidecar** (`doc-8`). Post-v1, tracked under m-3. The manifest schema it needs is
  REL-11, which is in M2.
- **Per-package versions** (`decision-7`). Needs its own decision before any monorepo can use
  the controller. Tracked under m-3.
- **Each further registry target** (`decision-9`). Each one needs its manifest and resume
  behaviour specified and tested first. Tracked under m-3.
- **Changesets.** Out of scope, because a second version authority is not allowed (`decision-6`).

## Open questions

Answered in `decision-10` and REL-9 (done): entrypoint is `pixi global install` from the project
channel, renderer scope is release lane only, `pixi-sandbox` and `pixi-release` own separate
workflow files with distinct markers, and v0.1.0 is clean-room rather than seeded from `pixi-sandbox`'s `xtask`.

## Corrections made

- `doc-3` §5 row 8 (`apps/docs/`) was removed. Row 4 (`pixi-sandbox init`) now names its owner.
- REL-2 to REL-7 each gained a checkable proof criterion, so their milestones can close.


## Intake of 2026-10-23 — GitHub issues #7–#10

Issues #7 and #8 are the same proposal at different levels of detail, so they are read together and
the tasks cite both. Every capability they asked for was checked against the decisions before a task
was created; three of the five did not need one, because the specification already covers them.

| Proposed in | Task | Outcome |
|---|---|---|
| #7 #8 §1 `pixi release reserve` | REL-13 → REL-14 | Gated: a reservation is a registry write, so `decision-9` holds it behind REL-6; the archive build stays a consumer `pixi run` task (`decision-2`) |
| #7 #8 §2 OIDC trusted publishing | REL-13 → REL-15 | Partly specified already: `doc-2` §3.2 renders `id-token: write` with an `environment:` boundary for the channel. The per-registry trust setup is gated |
| #7 #8 §3 polyglot two-phase build then publish | — | Already the design: `doc-2` §3.2 splits build, prove and publish with an artifact handoff, and `decision-2` keeps compilation in the repository. No task created |
| #7 #8 §4 GitHub Release assets and `SHA256SUMS` | — | Already specified: the `stage` job in `doc-2` §3.2 uploads assets with `SHA256SUMS`, `plan.json` and `evidence.json`, then reads them back. No task created |
| #7 #8 §5 scoped releases and milestone tag triggers | REL-16 | Conflicts with `decision-7` (one version, one tag) and `decision-4` (the annotated tag is the evidence anchor). A decision comes first |
| #9 issue forms, PR template, PR-title check | REL-17 → REL-18 | Outside the release lane (`decision-10` §2), so gated on a scope decision |
| #10 Starlight site, docs sync, Pages workflow | REL-19 → REL-20 | No public docs home exists (`doc-3` §1.4); gated |

Milestones: the registry and per-package work stays in **m-3**, as this document already says.
**m-4** was added for the governance and documentation surface, which is not a registry target and
not the release lane.

Version mapping is unchanged: nothing in this intake enters v0.2.0, v0.3.0 or v1.0.0.
