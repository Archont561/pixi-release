# `.knowledge/`

Durable context for `pixi-release` that is **not** a task and **not** canonical text.

## The split

| Lives in | Contains |
|---|---|
| `backlog/tasks/` | Actionable work. Created and edited **only** through the Backlog CLI. |
| `backlog/docs/` | Canonical specifications, research and spikes. |
| `backlog/decisions/` | Canonical rules, in ADR form. |
| `.knowledge/` | Pointers, design history, and facts about the working environment. |

**One canonical source.** A statement exists in exactly one place: its rule in a decision, its
specification in a document, its measurement in the research or spike document that produced
it. Other documents **cite** it — by Backlog id and section — and never restate it. A
`.knowledge/` file that has been promoted is reduced to a one-line pointer stub, never a second
copy.

The test for what gets promoted: **if someone would be wrong to contradict it, it is a Backlog
document or decision.**

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

Single canonical homes worth knowing: the `.pixi-release.toml` schema is `doc-3` §4; the action
SHA pin list is `doc-4` §7b; the command surface is `doc-2` §4; the rendered workflow files are
`doc-2` §3.

## What stays here, permanently

- `decisions.md` — how the design got here: the revision-2 defect ledger, and directions
  rejected before they reached an ADR. No decision is restated.
- `research/agent-environment.md` — what the agent sandbox can and cannot reach, and the CLI
  gotchas that follow. Not project knowledge, so it has no Backlog home.
- The pointer stubs.

## Conventions

- **Provenance tags.** ✅ = empirically measured. ⚠️ = reasoned assumption, still wants
  validation. Required on every claim in a research, spike or environment file; specification
  prose is unmarked because it is specification, not evidence.
- **Stable anchors.** Section numbers (`doc-2` §2.6) and decision ids (`decision-4`) are stable
  and may be cited from code comments and commit messages. Renumber nothing without fixing the
  citations.
- **Reference environment.** All ✅ measurements were taken on 2026-10-04 against
  `Archont561/*` default branches and GitHub-hosted runners; probe runs used
  `Archont561/pixi-release` on branch `arena/01a10687-pixi-release`.
- New decisions land through the Backlog CLI. They are **not** mirrored here.
