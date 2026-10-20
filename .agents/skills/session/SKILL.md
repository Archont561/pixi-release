---
name: session
description: Run a pixi-release work session start to finish — bring the environment up, survey the Backlog, propose the session and stop, work the tasks under AGENTS.md, then after the pull request merges report it and write the next session's opening prompt. Use at the start of any session on this repository, when the user says to initialize or bootstrap the environment, pick up tasks from the backlog, or asks what to work on; and again at the end, when work has landed or the user asks for a session report, a hand-off, or what the next session should start with.
---

# Session lifecycle for pixi-release

A session is a loop with five phases, and this file sequences all of them: **(1)** bring the
environment up, **(2)** survey the Backlog, **(3)** propose the session and stop, **(4)** work the
tasks under `AGENTS.md`, **(5)** after the pull request merges, report and hand the next session its
opening prompt. The four artifacts those phases produce have templates in
[`standup-template.md`](standup-template.md), cited below as *template §1* … *template §4*.

Two rules span the whole loop. Never propose work you cannot execute — an environment that is down
changes what is possible. And never close on an unproven claim: a criterion you could not verify is
an open item, not a checked box.

Authority: `AGENTS.md` governs. Where this skill and `AGENTS.md` disagree, `AGENTS.md` wins and this
file is wrong — fix it in the same commit.

## 1. Bring the environment up

1. **Confirm the branch.** `git status --short` and `git branch --show-current`. Work only on the
   session branch (`arena/<session-id>-pixi-release`). Never create, switch to, or push to another
   branch. Never rename the session branch.
2. **Sync.** `git fetch origin`. If the session branch is behind, fast-forward it; do not rebase or
   reset it. Note any uncommitted files and say whether they are yours.
3. **Toolchain.** Check what exists before promising anything:
   ```bash
   command -v pixi bun cargo gh; node -v
   ```
   - If `pixi` exists: `pixi run setup` (Bun install and Lefthook hooks), then `pixi run gates`
     as the baseline.
   - If `pixi`, `bun`, or `cargo` is missing (the sandbox measured this on 2026-10-04 — re-check,
     it may have changed): Backlog is still usable through `npx -y backlog.md@1.53.0 …`, the exact
     version pinned in `package.json`. Local gates are **not** runnable. Say so, and treat CI as the
     authority: `gh run list --branch <branch> --limit 5`.
4. **GitHub.** `gh auth status`. If it fails with an authentication error, tell the user the GitHub
   connection needs attention and ask them to reconnect GitHub in Arena. Do not ask for tokens.
5. **Environment facts.** Read `backlog/docs/research/agent-environment/doc-7` before planning any
   release probe. It lists what the sandbox cannot reach (Actions logs, release asset uploads) and
   the CLI gotchas.

Record the outcome in the standup (template §2): what is up, what is down, and what that rules out.

## 2. Survey the Backlog

Use the Backlog CLI. Do not hand-edit task Markdown (`AGENTS.md`).

```bash
B="npx -y backlog.md@1.53.0"      # or: pixi run backlog --   when pixi is present
$B task list --plain              # To Do / In Progress / Done, with AC counts
$B milestone list
$B board
$B search "<keyword>"             # before proposing anything, check for duplicates
$B doctor                         # duplicate ids, dependency cycles
$B task view REL-N --plain        # read the full task before proposing it
```

Read the decisions and documents a candidate task cites before proposing it. Decisions are the
rules: `backlog/decisions/`. Specifications are normative: `backlog/docs/`. Design history and
rejected directions: `doc-6`.

Check dependencies: a task whose `dependencies:` list has an unfinished task is not ready.

## 3. Propose the session, then stop

Write the proposal in the standup (template §2) and **stop for the user's go-ahead**. The proposal
states:

- the tasks you intend to move, with their ids and the acceptance criteria you expect to prove;
- anything you will *not* attempt, and why (environment, dependency, scope);
- the check you will run to prove each criterion (`pixi run gates` locally when available; CI
  otherwise).

Do not start implementation before the user confirms.

## 4. Work the tasks — house rules

The rules live in `AGENTS.md`. Read its *Rust boundaries*, *Release safety rules*, and *What not to
commit* sections before the first edit. The points most often broken:

- **Claim the task.** Set `In Progress` through `backlog task edit <id> -s "In Progress"`.
- **One canonical source.** A statement exists in exactly one place. Cite a decision or document by
  id and section; never restate it. If a conclusion would be wrong to contradict, promote it into a
  decision or document through the Backlog CLI, in the same change.
- **Command surface.** Repository work is `pixi run <verb>` → root package script →
  `bun x turbo run <verb>`. Do not add a second CI or hook path. Scope Turbo with
  `pixi run test -- --filter=@pixi-release/rust`.
- **Behaviour before code.** Write the failing test first, against public APIs and fixtures, not
  against this repository or a real `HOME`. Fake Git (`FakeGit`) proves policy before any shell
  integration.
- **Definition of done.** `pixi run gates` green (or the CI equivalent, named in the report), and
  every acceptance criterion verified against the code, not against memory. A public behaviour
  change also updates the matching specification in `backlog/docs/`. A design change needs a new
  decision, recorded before implementation, with any superseded decision marked.
- **Commits.** Conventional commits, enforced by `convco` through `lefthook` (`commit-msg`). One
  focused commit per task where practical. Do not rewrite pushed history.
- **Hand-off per task.** Write the task hand-off (template §3) when a task closes or stalls.

## 5. Merge, close, and report

Once the work is ready:

1. **Open the pull request** from the session branch to `main`, with a conventional title.
2. **Watch the checks.** `gh pr checks <number> --watch` or `gh run list --branch <branch>`. When a
   job fails, read its log before changing code: `gh run view <id>`; for the job log use
   `gh api repos/Archont561/pixi-release/actions/jobs/<job-id>/logs` (`gh run view --log` often
   cannot stream from this sandbox). Filter on a changed run id — `gh run list --limit 1` right
   after a push can return the previous run.
3. **Merge with a normal merge commit only.** `gh pr merge <number> --merge`. Never `--squash`,
   `--rebase`, or `--auto`. Run it only once the checks you care about are green, and name the
   number explicitly. The merge command must be the last command in its chain.
4. **Close the tasks in Backlog.** Every acceptance criterion verified; status moved with
   `backlog task edit <id> -s Done`; notes appended through the CLI. **An AC you could not prove is
   not a checked AC.** Leave that task `In Progress`, name the missing proof in its description
   (`backlog task edit <id> -d "…"`), and carry the same sentence into the report.
5. **Re-baseline.** `git fetch origin && git log --oneline origin/main -1`, then the gates on the
   merged tree. Put the result in the report.
6. **Record what you did not implement.** Proposals weighed and rejected, measurements worth keeping,
   and anything deferred go into Backlog — as a task (`backlog task create`) or into `doc-6` as a
   rejected direction. Do not leave them only in the chat, and do not add a new file to the
   repository for them.
7. **Report** (template §4), then write the opening prompt for the next session. The user should be
   able to paste that prompt into a fresh session and have the same understanding without reading the
   diff.

A session that merged nothing still reports: "Merged: nothing", what was recorded, and an opening
prompt that says what to decide first.
