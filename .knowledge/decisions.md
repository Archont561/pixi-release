# Decision history

Decisions themselves live in `backlog/decisions/` and the determinations they came from live in
`backlog/docs/`. **This file does not restate either.** It holds the one thing neither has room
for: how the design got here, and what was wrong with it before.

Index of current decisions: `README.md`.

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
