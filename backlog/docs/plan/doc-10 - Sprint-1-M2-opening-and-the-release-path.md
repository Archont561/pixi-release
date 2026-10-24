---
id: doc-10
title: Sprint 1 - M2 opening and the release path
type: guide
created_date: '2026-10-24 15:37'
updated_date: '2026-10-24 15:58'
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

Updated 2026-10-24: the missing link is now present. `decision-14` and REL-27 add the owner-lane
train, so option **B** from the first version of this plan was taken — the lane was built rather
than deferred, and the milestone-tag question was answered as a *dispatch trigger* rather than as a
second evidence anchor.

| Link | Status on 2026-10-24 |
|---|---|
| Version | ✅ `plan` derives it through Convco (`decision-6`); `plan --version` overrides it explicitly |
| Merge | ✅ normal merge commit, `gh pr merge --merge` |
| Tag | ✅ `autorelease.yml` writes the annotated `v*` tag (bootstrap, `decision-14` §3); `cut` takes it over in REL-7 |
| Trigger | ✅ a `milestone-v*` / `milestone/*` / `m-v*` tag or a dispatch *asks* for a release; a `main` push only reports the version it would get (`decision-14` §4, §9). The run dispatches `release.yml` because a `GITHUB_TOKEN` push starts nothing (`doc-3` §2.3 rule 1) |
| Release workflow | ✅ `release.yml` builds the conda artifact from the tagged source and proves it before publishing |
| Publish | ✅ channel `archont561/archont561` through the `prefix` OIDC binding; a GitHub Release carries the same bytes plus `SHA256SUMS` |

What is still missing, named rather than implied:

- **Platform breadth.** One platform (`linux-64`) until a build matrix exists (`decision-14` §8).
- **Self-release by a pinned controller.** REL-7 replaces the bootstrap tag step; until then the
  train tags with Convco and `git`, and the tag body is the changelog rather than `cut`'s trailers.
- **A proven release run.** The train has executed once and stopped at the packaging step
  (`REL-27`: the recipe had no `c_stdlib`); the artifact has not been built by CI yet, so the
  publish link below is written, not demonstrated.
- **Artifact verification and resumable publish** (REL-6) and the rendered consumer shapes
  (REL-5, REL-12). The owner-lane files are not their output (`doc-3` §2.1).

## 6. Stop conditions

Stop the sprint and report rather than improvise when:

- a task's proof needs a capability the sandbox does not have (for example a real registry publish,
  per `doc-7`), so its criterion cannot be checked here;
- a task turns out to contradict a decision — record the contradiction, do not route around it;
- the environment cannot be restored, so no gate can run locally and CI becomes the only authority;
- the release path in §5 is unresolved when the sprint reaches criterion 2.
