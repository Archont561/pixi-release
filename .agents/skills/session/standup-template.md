# Session templates — pixi-release

Five artifacts, produced at fixed points in the sprint loop. Use these shapes rather than inventing
new ones. Keep them plain: short lines, ids, and commands.

## Template §1 — Opening prompt

The first message of a session. It must be enough for a fresh agent to reach the same understanding
without reading the previous session. A session is sprint-long by default; say so here.

```
Repository: Archont561/pixi-release, main. Session branch: arena/<session-id>-pixi-release.
Read AGENTS.md and .agents/skills/session/SKILL.md first. This session is sprint-long unless I say otherwise.
Last merged: PR #<n> (<merge sha>). Baseline: <gates result or "CI only, local toolchain absent">.
Sprint: doc-<n> — <milestone id and name>. Tasks in order: <REL-a, REL-b, REL-c>.
Expected version: <x.y.z, or "none this sprint" and why>.
Open decision for you: <one question, or "none">.
```

## Template §2 — Sprint standup (start of session, phases 1–3)

```
## Standup — <yyyy-mm-dd>

Environment
- Branch: <branch>, behind/ahead origin: <n>/<n>
- pixi: <present|absent>   bun: <…>   cargo: <…>   gh auth: <ok|needs attention>
- Gates: <ran locally: pass/fail | not runnable here — CI is the authority>
- What this rules out: <…>

Backlog survey
- Milestone in scope: <m-n — name, <done>/<total>>
- In Progress: <ids>
- Ready To Do in this milestone: <ids>
- Blocked, and by what: <id — dependency in a later milestone>
- doctor: <clean | findings>

Sprint proposal (awaiting go-ahead)
- Exit condition: <what is true when the sprint closes, including the version>
- Order: <REL-a (prove AC <n,n>; check <command>)> → <REL-b …>
- Release: <cut --push at the end | blocked by decision-N, options A/B>
- Not attempting: <item — reason>
- Session length: sprint-long; task-long only if <named reason>
```

## Template §3 — Task hand-off (when a task closes or stalls mid-sprint)

```
## Hand-off — <REL-n> <title> — <status: Done | In Progress>

Acceptance criteria
- #1 <text> — proven by <command/test/CI run>
- #2 <text> — NOT PROVEN: <what is missing>

Changes: <commit shas, one line each>
Decisions or docs touched: <ids>
Open items: <…>
```

## Template §4 — Sprint report (after the sprint closes)

```
## Sprint report — <yyyy-mm-dd>

Sprint: doc-<n> — <milestone id and name>, <done>/<total> at close
Merged: PRs #<n>, #<n> — <titles> (merge commits <sha>)   |   or "Merged: nothing"
Post-merge checks: <ci run ids and verdicts; other lanes if triggered>
Tasks closed: <ids>   Tasks left In Progress: <ids — missing proof named>
Milestone: <closed | open, <n> task(s) remaining: ids>
Release: <version cut, tag and its push | refused: key and why | none, and why>
Re-baseline on origin/main: <gates result>
Recorded in Backlog: <new tasks, doc-10 update, rejected directions in doc-6>

Next sprint opening prompt:
<Template §1 block, filled in>
```

## Template §5 — Sprint plan document (`docs/plan`, created through the CLI)

A sprint's scope has no other home: the CLI cannot edit a milestone description (`doc-9`). Create
with `backlog doc create "<title>" -p plan -t guide`, then set the body with
`backlog doc update doc-<n> --content "$(cat body.md)"`.

```
## Status
<opened date; which milestone this sprint opens or closes; why the plan lives here>

## 1. What a sprint is here
<one milestone slice; one session; one PR per task; one release decision; written rollover>

## 2. Opening a sprint: the checklist
<the eight questions with the command behind each; name the two that actually reject sprints>

## 3. Sprint scope
<ordered table: task, milestone, why it is here, proof>  +  <deferred, with reasons>

## 4. Exit criteria
<numbered; includes the version and the milestone state>

## 5. The release path, and what it still needs
<link-by-link status: version, merge, tag, trigger, publish; name the decision that gates any
missing link, and give the honest options rather than assuming one>

## 6. Stop conditions
<when to stop and report instead of improvising>
```
