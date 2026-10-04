# `.knowledge/`

Durable context for `pixi-release` that is **not** a task and **not** a canonical specification.

## The split

| Lives in | Contains |
|---|---|
| `backlog/tasks/` | Actionable work. Created and edited **only** through the Backlog CLI. |
| `backlog/docs/` | Canonical specifications, research and spikes — the normative text. |
| `backlog/decisions/` | Decisions, in ADR form. The rule, not the reasoning behind it. |
| `.knowledge/` | Rationale, raw measurements, historical decisions, and discovery pointers. |

The test for where something belongs: **if someone would be wrong to contradict it, it is a
Backlog document or decision.** If it only explains *why* the canonical text says what it says,
or is a number somebody measured, it stays here.

When a document is promoted into Backlog, the `.knowledge/` file **becomes a pointer stub**
rather than a second copy. There is never a divergent fork of the same text.

## Canonical documents

| Backlog | Title | Pointer stub |
|---|---|---|
| `doc-1` | Release controller architecture | — (authored directly in Backlog) |
| `doc-2` | Consumer Release Workflow Specification | `consumer-workflow.md` |
| `doc-3` | Repository and Workflow Shape Specification | `repository-and-workflow-shape.md` |
| `doc-4` | Sibling Repository Delivery Survey | `sibling-repo-survey.md` |
| `doc-5` | Draft Release Visibility Spike | `draft-release-probe.md` |

| Backlog | Decision | Status |
|---|---|---|
| `decision-1` | Use Pixi → Bun → Turbo as the only quality-gate entrypoint | accepted |
| `decision-2` | `pixi-release` is a release controller, not a build system | proposed |
| `decision-3` | Generated workflows are marked, stamped, drift-gated, and never float a production pin | proposed |
| `decision-4` | The annotated tag is the release evidence anchor | proposed |

## Never promoted

These stay raw here, permanently:

- `research/EVIDENCE.md` — measurements, line counts, pinned SHAs, probe results. Numbers, not
  prose. A specification cites these; it does not restate them.
- `research/DRAFT-RELEASE-PROBE.md` — the run record for the `doc-5` spike, including the dead
  ends. Reproducibility, not conclusions.
- `decisions.md` — the running rationale log, including rejected options and the design history
  that produced the canonical text.

## Conventions

- **Provenance tags.** ✅ = empirically measured in this account or in a recorded run.
  ⚠️ = reasoned assumption, still wants validation. An untagged claim in an evidence file is a
  bug; untagged prose in a specification is specification, not evidence.
- **Stable anchors.** Section numbers (`doc-2 §2.6`) and decision ids (`decision-4`) are stable
  and may be cited from code comments and commit messages. Renumber nothing without fixing the
  citations.
- **Reference environment.** All ✅ measurements below were taken on 2026-10-04 against
  `Archont561/*` default branches and GitHub-hosted runners; probe runs used
  `Archont561/pixi-release` on branch `arena/01a10687-pixi-release`.
- New decisions land through Backlog decisions and are mirrored into `decisions.md`.
