---
id: doc-6
title: Design history and rejected directions
type: other
created_date: '2026-10-20 17:55'
---
## Purpose

How the design reached its current form: the defect ledgers from adversarial review, and directions
rejected before they reached a decision. Decisions live in `decision-1`–`decision-5`; the
determinations they came from live in `doc-2`–`doc-5`. This document does not restate either. Where
a fix is named here, the canonical statement is the cited section.

## Revision-2 defect ledger

The consumer specification (`doc-2`) reached its current form by adversarial review of its own
second revision. Seven real defects and three overclaims. Kept because the defects are easy to
reintroduce, and because the fix for each is now stated somewhere that no longer explains what it
was fixing.

| # | Defect in revision 2 | Fixed in |
|---|---|---|
| 1 | **The plan did not survive between workflows.** `prepare` uploaded `plan.json` as an artifact of a *different* run; `verify` then claimed to check "against the plan" with no plan in hand. The immutable plan was a recomputation that could silently differ. | `decision-4` |
| 2 | **The proof gated the GitHub Release but not the channel.** Channel upload happened before the proof, so the irreversible half was ungated. | `doc-2` §3.2 |
| 3 | **A target was a `(registry, platform)` pair**, inviting a channel that serves 3 of 5 platforms and calls it success. | `doc-2` §2.4 |
| 4 | **Resume rebuilt everything.** Re-dispatching re-ran the full matrix to publish one missing target. | `doc-2` §3.2, `doc-3` §2.6 |
| 5 | **Concurrency did not prevent two releases in flight.** `prepare` and `publish` used different groups, so v1.3.0 could be cut while v1.2.3 was half-published. GitHub concurrency cannot express a cross-workflow constraint. | `doc-2` §2.5 |
| 6 | **No rollback story at all.** | `doc-2` §6 |
| 7 | **Nothing let a user discover a wrong policy before cutting.** The first signal was a red release. | `doc-2` §1.2 |
| 8 | *Overclaim*: "draft assets are a real download, not simulated." True, but **authenticated** — tier 1 cannot prove public reachability. | `doc-2` §3.4, stated as a limit |
| 9 | *Overclaim*: `run: pixi run … -- $ARGS` presented as clean. It is unquoted word-splitting (SC2086). | `doc-2` §3.2 names the wart |
| 10 | *Unverified*: whether `contents: read` can list draft releases. Flagged honestly rather than asserted. | Measured — `doc-5` |

Defect 10 is the one worth remembering. It was the only item carried as an open question rather
than a claim, and measuring it found a **second, unanticipated** failure — drafts have no git tag,
so get-by-tag 404s even with write — that would have broken every draft lookup in production.

## Revision-3 defect

Found 2026-10-04 by grilling the sibling-GitHub-App proposal: `release-upgrade.yml` opened its PR
with `github.token`, which starts no CI run — so every upgrade PR would have landed reviewable but
unverified. That is the measured relock incident (`doc-4` §6.1) rebuilt into a brand-new workflow.
Fixed in `decision-5` (dispatch-and-attach by default, token-natural when `RELEASE_PUSH_TOKEN`
exists) and rendered in `doc-2` §3.3. The app proposal itself got a decision rather than a line
here, because it passed the promotion test: `decision-5`.

## Rejected directions

These were considered and dropped before they reached a decision, so no decision records them.

- **Absorbing `pixi-sandbox`'s pack → doctor → publish → restore proof cycle.** That is
  channel-publication machinery, not release-controller work; copying it would quietly re-import
  the mistake `decision-2` exists to prevent.
- **A composite `setup/action.yml` as the primary consumer entrypoint.** It would be a second
  version domain alongside the stamp, and the stamp *is* the pin. ⚠️ Revisit only with a
  measurement: five jobs each re-install the tool at roughly 10 s apiece.
- **A `--resume` flag.** It makes the user responsible for knowing what already happened, which is
  precisely the state the tool is holding on their behalf.
- **Per-`(registry, platform)` retry granularity.** Finer-grained, but it makes "is this released?"
  unanswerable.
- **Dropping Turbo to match `pixi-sandbox`.** Rejected in `decision-1`, recorded here only because
  the opposite choice is the one a reader will expect.

## Index of determinations

Titles and ids only. Follow the link for the rule; nothing is summarised here, because a summary is
a second copy that goes stale. Decisions are listed by `backlog decision list`.

| Decision | Title | Status |
|---|---|---|
| `decision-1` | Use Pixi → Bun → Turbo as the only quality-gate entrypoint | accepted |
| `decision-2` | `pixi-release` is a release controller, not a build system | accepted |
| `decision-3` | Generated workflows are marked, stamped, drift-gated, and never float a production pin | proposed |
| `decision-4` | The annotated tag is the release evidence anchor | accepted |
| `decision-5` | No sibling GitHub App — the release push-token slot is consumer-owned | proposed |

Determinations that are canonical in a document rather than a decision:

| Determination | Canonical |
|---|---|
| Two release workflows: `cut` ≠ `publish` | `doc-2` §2.1, §3 |
| A target is a registry, not a `(registry, platform)` pair | `doc-2` §2.4 |
| One release at a time is a precondition, not a concurrency group | `doc-2` §2.5 |
| Draft releases constrain the proof job | `doc-2` §2.6, `doc-5` |
| Proof runs before channel upload | `doc-2` §3.2 |
| Consumer proof owns half the problem, deliberately | `doc-2` §3.4 |
| The command surface | `doc-2` §4 |
| Abandon and yank | `doc-2` §6 |
| Single-root Cargo workspace; grow crates on triggers | `doc-3` §1.2 |
| `crates/xtask` is strictly owner-lane, with a deletion trigger | `doc-3` §1.3 |
| `crates/xtask/fixtures/consumers/` is a first-class deliverable | `doc-3` §1.5 |
| The workflow set to own, and the nine renderer behaviours | `doc-3` §2.2, §2.3 |
| Delete the hand-maintained `git add` list | `doc-3` §2.5 |
| Resume is an artifact contract, not a flag | `doc-3` §2.6 |
| The `.pixi-release.toml` schema | `doc-3` §4 |
| Action SHA pin list | `doc-4` §7b |

Single canonical homes worth knowing: the `.pixi-release.toml` schema is `doc-3` §4; the action SHA
pin list is `doc-4` §7b; the command surface is `doc-2` §4; the rendered workflow files are `doc-2` §3.

## Conventions

- **Stable anchors.** Section numbers (`doc-2` §2.6) and decision ids (`decision-4`) are stable and
  may be cited from code comments and commit messages. Renumber nothing without fixing the citations.
- **Provenance tags.** ✅ = empirically measured. ⚠️ = reasoned assumption, still wants validation.
  Required on every claim in a research, spike or environment document; specification prose is
  unmarked because it is specification, not evidence.
- New decisions land through the Backlog CLI. They are not mirrored anywhere else.

## Removals on 2026-10-20

- **`apps/docs/`** was removed, with its Bun workspace entry, its lock entry, and its checks in `verify-workspace.ts` and `xtask`. Public documentation has no home in this repository until one is decided.
- **`.pixi-release.toml`** (this repository's own authored policy) was removed. The schema is still specified in `doc-3` §4, and the consumer fixtures in `crates/xtask/fixtures/consumers/` still carry the policy shape. Restore the file when the policy parser (REL-2) lands.
