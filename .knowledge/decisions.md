# Decisions — index and history

Decisions themselves live in `backlog/decisions/` and the determinations they came from live in
`backlog/docs/`. **This file does not restate either.** It is an index, plus the one thing
neither has room for: how the design got here, and what was wrong with it before.

## Index

Titles and ids only. Follow the link for the rule; nothing is summarised here, because a
summary is a second copy that goes stale.

| | Decision | Date | Status |
|---|---|---|---|
| `decision-1` | Use Pixi → Bun → Turbo as the only quality-gate entrypoint | 2026-10-01 | accepted |
| `decision-2` | `pixi-release` is a release controller, not a build system | 2026-10-04 | proposed |
| `decision-3` | Generated workflows are marked, stamped, drift-gated, and never float a production pin | 2026-10-04 | proposed |
| `decision-4` | The annotated tag is the release evidence anchor | 2026-10-04 | proposed |
| `decision-5` | No sibling GitHub App — the release push-token slot is consumer-owned | 2026-10-04 | proposed |

Determinations that are canonical in a document rather than a decision, and where they live:

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
| `tests/fixtures/consumers/` is a first-class deliverable | `doc-3` §1.5 |
| The workflow set to own, and the nine renderer behaviours | `doc-3` §2.2, §2.3 |
| Delete the hand-maintained `git add` list | `doc-3` §2.5 |
| Resume is an artifact contract, not a flag | `doc-3` §2.6 |
| The `.pixi-release.toml` schema | `doc-3` §4 |
| Action SHA pin list | `doc-4` §7b |

---

## The revision-2 defect ledger

The consumer specification reached its current form by adversarial review of its own second
revision. Seven real defects and three overclaims. Kept because the defects are easy to
reintroduce, and because the fix for each is now stated somewhere that no longer explains what
it was fixing.

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
than a claim, and measuring it found a **second, unanticipated** failure — drafts have no git
tag, so get-by-tag 404s even with write — that would have broken every draft lookup in
production.

## The revision-3 defect

Found 2026-10-04 by grilling the sibling-GitHub-App proposal: `release-upgrade.yml` opened its
PR with `github.token`, which starts no CI run — so every upgrade PR would have landed
reviewable but unverified. That is the measured relock incident (`doc-4` §6.1) rebuilt into a
brand-new workflow. Fixed in `decision-5` (dispatch-and-attach by default, token-natural when
`RELEASE_PUSH_TOKEN` exists) and rendered in `doc-2` §3.3. The app proposal itself got an ADR
rather than a line below, because it passed the promotion test: `decision-5`.

## Rejected directions, with no decision of their own

These were considered and dropped before they reached a decision, so no ADR records them.

- **Absorbing `pixi-sandbox`'s pack → doctor → publish → restore proof cycle.** That is
  channel-publication machinery, not release-controller work; copying it would quietly
  re-import the mistake `decision-2` exists to prevent.
- **A composite `setup/action.yml` as the primary consumer entrypoint.** It would be a second
  version domain alongside the stamp, and the stamp *is* the pin. ⚠️ Revisit only with a
  measurement: five jobs each re-install the tool at roughly 10 s apiece.
- **A `--resume` flag.** It makes the user responsible for knowing what already happened, which
  is precisely the state the tool is holding on their behalf.
- **Per-`(registry, platform)` retry granularity.** Finer-grained, but it makes "is this
  released?" unanswerable.
- **Dropping Turbo to match `pixi-sandbox`.** Rejected in `decision-1`, recorded here only
  because the opposite choice is the one a reader will expect.
