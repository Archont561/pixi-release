# Session templates — pixi-release

Four artifacts, produced at fixed points in the session loop. Use these shapes rather than
inventing new ones. Keep them plain: short lines, ids, and commands.

## Template §1 — Opening prompt

The first message of a session. It must be enough for a fresh agent to reach the same
understanding without reading the previous session.

```
Repository: Archont561/pixi-release, main. Session branch: arena/<session-id>-pixi-release.
Read AGENTS.md and .agents/skills/session/SKILL.md first.
Last merged: PR #<n> (<merge sha>). Baseline: <gates result or "CI only, local toolchain absent">.
Open decision for you: <one question, or "none">.
Proposed tasks: <ids and one-line goals>.
```

## Template §2 — Standup (start of session, phase 1–3)

```
## Standup — <yyyy-mm-dd>

Environment
- Branch: <branch>, behind/ahead origin: <n>/<n>
- pixi: <present|absent>   bun: <…>   cargo: <…>   gh auth: <ok|needs attention>
- Gates: <ran locally: pass/fail | not runnable here — CI is the authority>
- What this rules out: <…>

Backlog survey
- In Progress: <ids>
- Ready To Do (dependencies met): <ids>
- Blocked: <id — blocked by …>
- doctor: <clean | findings>

Proposal (awaiting go-ahead)
- <REL-n> — <goal>; will prove: <AC numbers>; check: <command or CI job>
- Not attempting: <item — reason>
```

## Template §3 — Task hand-off (when a task closes or stalls)

```
## Hand-off — <REL-n> <title> — <status: Done | In Progress>

Acceptance criteria
- #1 <text> — proven by <command/test/CI run>
- #2 <text> — NOT PROVEN: <what is missing>

Changes: <commit shas, one line each>
Decisions or docs touched: <ids>
Open items: <…>
```

## Template §4 — Session report (after the work lands)

```
## Session report — <yyyy-mm-dd>

Merged: PR #<n> — <title> (merge commit <sha>)   |   or "Merged: nothing"
Post-merge checks: <ci run ids and verdicts; docs/other lanes if triggered>
Tasks closed: <ids>   Tasks left In Progress: <ids — missing proof named>
Re-baseline on origin/main: <gates result>
Recorded in Backlog: <new tasks, rejected directions in doc-6>

Next session opening prompt:
<the Template §1 block, filled in>
```
