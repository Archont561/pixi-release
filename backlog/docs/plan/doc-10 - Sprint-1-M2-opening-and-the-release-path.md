---
id: doc-10
title: Sprint 1 - M2 opening and the release path
type: guide
created_date: '2026-10-24 15:37'
updated_date: '2026-10-24 15:37'
---
## Status

Opened 2026-10-24. This document is the operating plan for **Sprint 1**, which opens the next
milestone, **m-2** (M2 — Verify, publish, resume, and self-host), and closes the two m-1 tasks that
m-2 depends on. It lives here because the Backlog CLI cannot edit a milestone description (`doc-9`),
so sprint scope has no other home.

A sprint is **not** a second delivery authority. Every row below is an existing Backlog task with
its own criteria; this document only decides what is in the sprint, in what order, and what "done"
means at the end.

## 1. What a sprint is here

- **One milestone slice.** A sprint names a milestone and a subset of its tasks. It never adds work
  that is not already a task.
- **One session, one PR per task.** A session works the sprint; each task lands as its own pull
  request and its own merge commit, so a task that stalls does not hold the rest.
- **One release decision at the end.** A sprint ends by cutting the version the milestone earns, or
  by recording why it does not. It never ends with "the version is whatever Convco says later".
- **A written rollover.** Unfinished tasks stay `In Progress` with the missing proof named; the next
  sprint's plan inherits them explicitly.

## 2. Opening a sprint: the checklist

Every item is a question with a command behind it, because a sprint that cannot answer these is a
wish list.

| # | Question | Evidence |
|---|---|---|
| 1 | Does the milestone exist? | `backlog milestone list` |
| 2 | Does every task in scope carry its id? | `backlog task list --json`, filtered on `milestone` |
| 3 | Is every dependency finished? | `backlog task view REL-n` → *Dependency Graph* |
| 4 | Is the base green? | `pixi run gates` on `origin/main` |
| 5 | Does any planned claim contradict a decision? | `backlog decision list`, then the cited decision |
| 6 | Does every task carry a proof? | its acceptance criteria name a command or a CI job |
| 7 | Is the exit condition written? | §4 of this document |
| 8 | Is the release path decided? | §5 of this document |

Items 3 and 8 are the two that actually reject sprints. A task whose dependency sits in a later
milestone cannot start; a sprint that assumes a release trigger which does not exist ends at a tag
and no further (see §5).

## 3. Sprint 1 scope

Ordered. The order is the contract; the count is the session budget.

| # | Task | Milestone | Why it is here | Proof |
|---|---|---|---|---|
| 1 | REL-8 — Generate the changelog through Convco | m-1 | The v0.2.0 release commit must carry the changelog the policy names (`decision-6`) | gates on the merge commit; a `prepare` run stages the generated file |
| 2 | REL-5 — Render and verify release workflow templates | m-1 | REL-12 depends on it, and it is the only producer of the release lane | gates on the merge commit; golden fixtures |
| 3 | REL-11 — Define the release manifest schema | m-2 | Unblocked since REL-4 landed; REL-6 cannot record evidence without it | gates on the merge commit |
| 4 | REL-12 — Gate generated-workflow drift | m-2 | Depends on REL-5; it is what makes a rendered workflow trustworthy | gates on the merge commit |

**Deferred to Sprint 2, with the reason:**

- **REL-6** — depends on REL-11 (in this sprint) and on a target design that `decision-9` keeps
  behind it. Starting it before REL-11 finishes would invent the evidence format twice.
- **REL-7** — depends on REL-6 and REL-5. It is the N−1 self-release: the milestone's hard part,
  and the first task that needs a *published* prior version to pin.

Measured on 2026-10-24: m-1 stands at 4/6 (REL-5 and REL-8 open), m-2 at 0/4, and m-2's REL-12 is
blocked by REL-5 while REL-6 is blocked by REL-11. That is why the sprint opens with m-1's remainder
rather than m-2's first row.

## 4. Exit criteria

The sprint is done when all four hold:

1. **m-1 closes.** REL-5 and REL-8 are `Done`, each proven by a green `pixi run gates` on its own
   merge commit.
2. **v0.2.0 is cut.** `plan` → `prepare` → `cut --push` on `main`, producing an annotated tag whose
   body carries `pixi-release-version`, `plan-digest` and `policy-digest` (`decision-4`).
3. **m-2 records 2/4.** REL-11 and REL-12 `Done`; REL-6 and REL-7 remain, named as Sprint 2's
   opening work.
4. **The report exists.** Session report plus the next sprint's opening prompt, in the shape
   `.agents/skills/session/standup-template.md` prescribes.

If criterion 2 cannot be met, criterion 1 still closes the milestone, and the report states which
refusal key from `cut` stopped it. A refused cut is a result, not a failure.

## 5. The release path, and what it still needs

The intended flow — *one sprint produces a version, a pull request, a merge, and a release that
runs* — is five links. Four exist or are in scope; one does not.

| Link | Status on 2026-10-24 |
|---|---|
| Version | ✅ `plan` derives it through Convco (`decision-6`); `plan --version` overrides it explicitly |
| Merge | ✅ normal merge commit, `gh pr merge --merge` |
| Tag | ✅ `cut` writes the annotated tag after the merge (`decision-4`); it is the evidence anchor |
| Release workflow | ❌ **absent.** `.github/workflows/` holds `ci.yml`, `publish-sandbox.yml` and `relock.yml` only. The release lane is rendered by REL-5 and drift-gated by REL-12; the self-release is REL-7 |
| Publish | ❌ REL-6. Nothing can be published, and `decision-9` keeps registry targets behind it |

The missing link is not the tag, it is the **trigger**. A milestone tag that starts a release is a
new rule, and two accepted decisions stand in its way:

- `decision-4` makes the annotated tag the evidence anchor, written by `cut`, not by a merge.
- `decision-7` fixes one version and one tag per repository for v1.

REL-16 owns exactly this question — milestone tags classified as a release trigger or as a quality
gate — and its criterion 5 forbids implementation under that task. It sits in m-3, which is
post-v1. So there are two honest ways to end Sprint 1:

**A. End at the tag (no new decision).** `cut --push` publishes `v0.2.0` as the evidence anchor; the
milestone is recorded *inside* that tag's body rather than as a second tag. The release lane, when
REL-5 and REL-7 land, runs from the tag `cut` already writes. Nothing contradicts `decision-4` or
`decision-7`.

**B. Pull REL-16 forward and answer it first.** A `milestone-*` tag becomes a trigger, which needs a
decision that supersedes or amends `decision-7`, and an owner-lane release workflow that exists to
be triggered. This is the only route to "merge a milestone tag and watch a release run" during
Sprint 1, and it spends sprint capacity on a decision plus a workflow that REL-5 and REL-7 would
otherwise produce.

**Recommendation: A.** It keeps the sprint's exit condition reachable and defers the trigger
question to the task that was created for it. B is legitimate, but it is a scope change to m-3 work
and it should be an explicit choice, not an assumption buried in a plan.

## 6. Stop conditions

Stop the sprint and report rather than improvise when:

- a task's proof needs a capability the sandbox does not have (for example a real registry publish,
  per `doc-7`), so its criterion cannot be checked here;
- a task turns out to contradict a decision — record the contradiction, do not route around it;
- the environment cannot be restored, so no gate can run locally and CI becomes the only authority;
- the release path in §5 is unresolved when the sprint reaches criterion 2.
