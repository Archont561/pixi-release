---
id: doc-9
title: Roadmap
type: specification
created_date: '2026-10-20 19:06'
---

## Status

Proposal, written 2026-10-20 to replace the milestone outline in `doc-3` §5. It changes no task
and no milestone file: the Backlog CLI cannot edit a milestone description. The versions below
are expectations. Convco computes the real version from commit history (`decision-6`).

## Version mapping

| Version | Milestone | Exit condition |
|---|---|---|
| v0.1.0 | M0 | `pixi run ci` green from a fresh clone, verified on `main` (not yet proven) |
| v0.2.0 | M1 | A dry run proves why a tag may or may not be cut, without touching Git state |
| v0.3.0 | M2 (publish part) | Every required target has deterministic evidence and a recoverable publication path |
| v1.0.0 | M2 (self-host part) | The N−1 self-release is verified and the consumer proof passes (REL-7) |

Only `github-release` is an admissible target before REL-6 is done (`decision-9`). The first
tagged release that exercises a registry target is after that.

## Revisions to the existing milestones

- **M0.** Keep the scope. Its exit condition has not been demonstrated: CI has never completed
  green on this branch. Close M0 only with a green run, not with the task alone.
- **M1.** Add REL-5's renderer half (templates, golden fixtures, the three consumer shapes).
  Add changelog generation through Convco (`decision-6`).
- **M2.** Take REL-5's drift gate (markers, stamps, `workflow check`) and REL-6 (evidence JSON,
  per-target resume, the five-platform matrix, OIDC publish). Keep REL-7 as the last item.
- **REL-5 placement.** `doc-3` §5 puts renderer work in M1 (row 9) and drift work in M2
  (row 10). Split REL-5 into two tasks, one per milestone, rather than keeping one task that
  spans both.

## Scope outside the milestones

These are real work, deliberately not in v1:

- **Release manifest and presentation sidecar** (`doc-8`). Decide the manifest schema before
  REL-6 starts, because REL-6's evidence JSON is the natural base for it. The sidecar itself
  is post-v1.
- **Per-package versions** (`decision-7`). Needs its own decision before any monorepo can use
  the controller.
- **Each further registry target** (`decision-9`). Each one needs its manifest and resume
  behaviour specified and tested first.
- **Changesets**. Out of scope, because a second authority is not allowed (`decision-6`).

## Unanswered questions that block M1

From `doc-3` §6, still open: the consumer entrypoint (composite action or package install), whether
the renderer owns `ci.yml` and `docs.yml`, the relationship to `pixi-sandbox`'s markers and
bots, and whether to seed from `pixi-sandbox`'s `xtask`. M1 cannot size its tasks until these
are answered.

## Corrections to existing documents

- `doc-3` §5 row 8 plans `apps/docs/`, which was removed on 2026-10-20.
- `doc-3` §5 row 4 (`pixi-sandbox init`) has no task in the Backlog.
- REL-2 to REL-7 each have three acceptance criteria and no proof of completion beyond their
  descriptions. Each one needs a checkable criterion before its milestone can close.
