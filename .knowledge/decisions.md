# Decision log

Running rationale log. Each entry records *why*, what was rejected, and where the normative
text lives. **New decisions land through Backlog decisions and are mirrored here** — where a
`backlog/decisions/decision-N` exists it is the rule; this file is the reasoning.

Entries D5 onward are design determinations that are canonical in a Backlog **document** rather
than a decision. They are logged here so the reasoning is not lost inside a long specification.

| | Decision | Date | Canonical | Status |
|---|---|---|---|---|
| D1 | Pixi → Bun → Turbo is the only quality-gate entrypoint | 2026-10-01 | `decision-1` | accepted |
| D2 | `pixi-release` is a controller, not a build system | 2026-10-04 | `decision-2` | proposed |
| D3 | Generated files are marked, stamped, drift-gated, never float | 2026-10-04 | `decision-3` | proposed |
| D4 | The annotated tag carries the evidence anchor | 2026-10-04 | `decision-4` | proposed |
| D5 | Two release workflows: `cut` ≠ `publish` | 2026-10-04 | `doc-2` §2.1, §3 | proposed |
| D6 | A target is a registry, not a `(registry, platform)` pair | 2026-10-04 | `doc-2` §2.4 | proposed |
| D7 | One release at a time is a precondition, not a concurrency group | 2026-10-04 | `doc-2` §2.5 | proposed |
| D8 | Proof runs before channel upload | 2026-10-04 | `doc-2` §3.2 | proposed |
| D9 | Resume is an artifact contract, not a flag | 2026-10-04 | `doc-3` §2.6 | proposed |
| D10 | `prove` holds no token and tests the workflow artifacts | 2026-10-04 | `doc-2` §2.6 | proposed |
| D11 | Drafts are addressed by release ID, never get-by-tag | 2026-10-04 | `doc-2` §2.6 | proposed |
| D12 | Consumer proof owns half the problem, deliberately | 2026-10-04 | `doc-2` §3.4 | proposed |
| D13 | Delete the hand-maintained `git add` list | 2026-10-04 | `doc-3` §2.5 | proposed |
| D14 | Ship `pixi release rehearse` | 2026-10-04 | `doc-2` §1.2 | proposed |
| D15 | Single-root Cargo workspace; grow crates on triggers | 2026-10-04 | `doc-3` §1.2 | proposed |
| D16 | Add `crates/xtask`, strictly owner-lane | 2026-10-04 | `doc-3` §1.3 | proposed |
| D17 | Reject a composite action as the primary consumer entrypoint | 2026-10-04 | `doc-2` §0 | proposed |

---

### D1 — Pixi → Bun → Turbo is the only quality-gate entrypoint

By `pixi-sandbox`'s own deferral triggers, Turbo is not yet earned here. Kept anyway because
the renderer must be correct for Turbo-shaped *and* non-Turbo consumers, and dogfooding the
more complex shape is the cheaper mistake. **Rejected:** dropping Turbo to match
`pixi-sandbox`; the renderer would then be written by people who had never run the path two of
five peers depend on. The exception is bounded by two written obligations — see `decision-1`.

### D2 — `pixi-release` is a controller, not a build system

An early draft rendered `pixi-release build --target <triple>`, which would have made the
controller own five toolchains. **Rejected:** a renderer growing a code path per artifact kind
(Rust binary, wheel, npm tarball, conda package) — unbounded work, and every new consumer shape
becomes a release of `pixi-release`. ✅ The decisive evidence is `geoaiworkbench`: it publishes
Python wheels and has no Cargo workspace, so a Rust-shaped controller is wrong on day one.

### D3 — Generated files are marked, stamped, drift-gated, never float

Adopted from `pixi-sandbox`'s `decision-4` rather than re-derived, with one addition: a
`# pixi-release-policy: sha256:…` digest, because the version stamp alone lets a config edit
silently desynchronise policy from the workflows enforcing it. **Rejected:** floating the
workflow to `@v0` (a new release changes consumer CI with no review), and leaving upgrades
manual and docs-only. ✅ `nadir/.github/workflows/publish-sandbox.yml` is what option (b) looks
like after a year: a 59-line unstamped fork of a 292-line generated file.

### D4 — The annotated tag carries the evidence anchor

The plan has to survive between two workflow runs, possibly days apart. **Rejected:**
committing `.pixi-release/plan.json` (`AGENTS.md` forbids committing that directory);
cross-workflow artifact download (expires in 7 days, exactly when resume matters); attaching
the plan to the draft release (does not exist at cut time, and cannot anchor the tag itself).
✅ `git tag -a` trailers are unused across the whole account, and `pixi-sandbox` already writes
annotated tags with empty messages — the space is free.

The real cost is that `cut` must now be deterministic in the same sense the renderer is. That
is deliberate: it turns an asserted immutability into a tested one.

### D5 — Two release workflows: `cut` ≠ `publish`

Cutting is cheap, reversible and fast; publishing is slow, irreversible and needs five runners.
Fusing them means a channel-upload failure forces a re-cut. Splitting them is also what makes
D4 necessary, and what makes resume (D9) expressible at all. Consequence worth remembering: a
tag pushed with `GITHUB_TOKEN` starts no workflow, so `prepare` must end with an explicit
`gh workflow run` dispatch. ✅ Three shipped `pixi-sandbox` bugs trace to this rule.

### D6 — A target is a registry, not a `(registry, platform)` pair

Revision 2 modelled targets as pairs, which invites a channel that serves 3 of 5 platforms and
calls it success. Platform completeness is now a `verify` precondition and the upload is one
atomic call per registry. **Rejected:** per-pair retry granularity — finer, but it makes
"released" unanswerable.

### D7 — One release at a time is a precondition, not a concurrency group

GitHub concurrency cannot express a cross-workflow constraint, so two groups would let v1.3.0
be cut while v1.2.3 is half-published. `plan` refuses while an unpromoted draft exists.
**Rejected:** a shared concurrency group — it would cancel or queue rather than refuse, and the
user would learn about the conflict from a queued job instead of an error.

### D8 — Proof runs before channel upload

Revision 2 uploaded to the channel and *then* proved, leaving the irreversible half ungated.
Order is now `triage → build → stage → prove → publish → promote`.

### D9 — Resume is an artifact contract, not a flag

Re-dispatching the same tag resumes. `triage` reads draft state and emits `need-build` /
`remaining`; the matrix is skipped when assets are already staged and verified, and `publish`
skips every registry with success evidence. **Rejected:** a `--resume` flag — it makes the user
responsible for knowing what already happened, which is precisely the state the tool holds.

### D10 — `prove` holds no token and tests the workflow artifacts

Forced by the spike (`doc-5`): `contents: read` cannot see a draft, and fails silently. The
obvious repair — widen `prove` to `contents: write` — would gut the point of isolating it.
Moving the clean room onto the workflow artifacts yields `permissions: {}`, which is a
*stronger* clean room than the design had before the measurement. The upload-integrity half
became a checksum read-back inside `stage`, where write already exists; it is a comparison, not
an execution, so it never wanted a clean room. **This is the entry to cite when someone asks
why a probe was worth the trouble: it improved the design rather than merely constraining it.**

### D11 — Drafts are addressed by release ID, never get-by-tag

Also forced by the spike, and unanticipated: a draft creates no git tag, so
`GET /releases/tags/{tag}` 404s **even with write**. `gh` hides this with a list-and-match
fallback; `pixi-release` is Rust calling REST directly and gets no fallback. Every
tag-addressed draft lookup in the previous design would have failed in production. The forge
port's `find_draft_by_tag_name()` is list + filter, with a fake-backed regression test.

### D12 — Consumer proof owns half the problem, deliberately

Tier 1 proves: our own download, `SHA256SUMS`, a clean room (`env -i`, scratch `HOME`, minimal
`PATH`), and `--version` agreeing with the tag. It does **not** prove public reachability —
a draft is invisible anonymously, so tier-1 proof is authenticated by construction. The limit
is stated rather than papered over. **Rejected:** absorbing `pixi-sandbox`'s
pack → doctor → publish → restore proof cycle; that is channel-publication machinery, not
release-controller work, and copying it would quietly re-import D2's mistake.

### D13 — Delete the hand-maintained `git add` list

✅ `geoquery` and `qgis-rs` `autorelease.yml` each carry a hand-maintained
`git add <10 explicit manifests>` with a `version-check` step as a band-aid for when the list
goes stale. Replacement: `prepare` records the touched set, `cut` stages exactly that set and
refuses any unaccounted modification. The `.release-touched` file in the same repositories is
the precedent that this is the shape the author already reaches for.

### D14 — Ship `pixi release rehearse`

Nothing in the design let a user discover a misconfigured `[build].task` before cutting; the
first signal was a red release. `rehearse` runs the whole lifecycle offline against a bare
remote and fake registries, reusing `doc-1`'s `FakeGit`.

### D15 — Single-root Cargo workspace; grow crates on triggers

Three crates today (`core`, `git`, CLI). New crates are justified by a trigger — a second
consumer of the code, or a dependency the CLI must not carry — not by anticipation.

### D16 — Add `crates/xtask`, strictly owner-lane

`pixi-release`'s own release must not be performed by the binary under test. `xtask` is the
owner lane; the rendered workflows are the consumer lane. Keeping them separate is what stops
a broken `pixi-release` from being unable to ship its own fix.

### D17 — Reject a composite action as the primary consumer entrypoint

A composite `setup/action.yml` would be a second version domain alongside the stamp, and the
stamp *is* the pin. ⚠️ Revisit only with a measurement: six jobs each re-install the tool at
roughly 10 s apiece. Measure before optimising.

---

## Design history

The consumer specification reached its current form by adversarial review of its own second
revision. Seven real defects and three overclaims, kept here because the defects are the
reasoning behind D4–D14 and are easy to reintroduce.

| # | Defect in revision 2 | Where it was fixed |
|---|---|---|
| 1 | **The plan did not survive between workflows.** `prepare` uploaded `plan.json` as an artifact of a *different* run; `verify` then claimed to check "against the plan" with no plan in hand. The immutable plan was a recomputation that could silently differ. | D4 / `decision-4` |
| 2 | **The proof gated the GitHub Release but not the channel.** Channel upload happened before the proof, so the irreversible half was ungated. | D8 |
| 3 | **A target was a `(registry, platform)` pair**, inviting a channel serving 3 of 5 platforms. | D6 |
| 4 | **Resume rebuilt everything.** Re-dispatching re-ran the full matrix to publish one missing target. | D9 |
| 5 | **Concurrency did not prevent two releases in flight.** `prepare` and `publish` used different groups. | D7 |
| 6 | **No rollback story at all.** | `doc-2` §6 — `abandon` / `yank` |
| 7 | **Nothing let a user find out their policy was wrong before cutting.** | D14 |
| 8 | *Overclaim*: "draft assets are a real download, not simulated." True, but **authenticated** — tier 1 cannot prove public reachability. | D12, stated as a limit |
| 9 | *Overclaim*: `run: pixi run … -- $ARGS` presented as clean. It is unquoted word-splitting (SC2086). | `doc-2` §3.2 names the wart and its mitigation |
| 10 | *Unverified*: whether `contents: read` can list draft releases. | Measured — `doc-5`, D10, D11 |

Defect 10 is the one worth remembering. It was flagged honestly as unverified rather than
asserted, then measured — and the measurement found a *second*, unanticipated failure (D11)
that would have broken every draft lookup in production.
