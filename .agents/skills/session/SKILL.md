---
name: session
description: Run a pixi-release work session start to finish. A session is sprint-long by default — bring the environment up, pick the milestone slice, propose the sprint and stop, work its tasks under AGENTS.md, then close the sprint with merges, a milestone check, a release decision and a hand-off for the next sprint. Use at the start of any session on this repository, and again at the end, when work has landed or the user asks for a sprint report or what the next sprint should start with. Shorten a session to one task only when the user asks for that, or when the work is dominated by external waits (many workflow runs, a relock round-trip, a release probe); name the reason and get the user's confirmation first.
---

# Sprint lifecycle for pixi-release

A session is **one sprint**, not one task. A sprint is a milestone slice: a subset of the tasks in
one Backlog milestone, worked in a fixed order, ending in merged pull requests, a closed edge of the
milestone, and a release decision. `doc-10` is the plan for the current sprint; when a sprint closes,
the plan for the next one is written the same way.

The loop has five phases: **(1)** bring the environment up, **(2)** survey the Backlog and pick the
sprint slice, **(3)** propose the sprint and stop, **(4)** work its tasks under `AGENTS.md`,
**(5)** close the sprint — merge, close tasks, check the milestone, decide the release, report, and
write the next sprint's opening prompt.

Three rules span the whole loop. Never propose work you cannot execute — a down environment changes
what is possible. Never close on an unproven claim — a criterion you could not verify is an open
item, not a checked box. Never let a sprint end without saying what version it produced, or why it
produced none.

Authority: `AGENTS.md` governs. Where this skill and `AGENTS.md` disagree, `AGENTS.md` wins and this
file is wrong — fix it in the same commit.

## 0. How long a session runs

| Mode | When | What it means |
| :-- | :-- | :-- |
| **Sprint-long** (default) | Always, unless one of the cases below applies | Work every task in the sprint plan, in order, until the sprint's exit criteria are met or a stop condition fires |
| **Task-long** | The user asks for a single task ("just do REL-8"), *or* the work is dominated by external waits | One task, one pull request, one report; the rest of the sprint is explicitly carried forward |

Switch to task-long only when the user asks for it, or when you can name the reason — many workflow
runs, a relock round-trip, a release probe against a real registry, an external review. **Say the
reason and confirm before shortening the session**; do not silently deliver one task of a five-task
sprint. Conversely, when the user asks for a task-long session, do not expand it back into the
sprint.

A sprint ships **one version**. If the sprint produced work that should be released, the release is
part of closing the sprint (§5.5), not a favour for a later session.

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
   - If `pixi`, `bun`, or `cargo` is missing, that is the fresh-sandbox state, not a dead end:
     **run the repository's own restore script.** `scripts/restore.sh` is the generated
     `pixi-sandbox` entry point (pixi-sandbox 0.8.0), and it resolves the branch, fetches it,
     unpacks the packed environment, verifies every file against the manifest and wires the
     vendored Cargo registry, so the gates afterwards need no network:

     ```bash
     # Restore into a scratch clone, never into the workspace: the unpacked environment is
     # ~1.7 GiB and the vendor tree another ~270 MiB, which would swamp the session snapshot.
     git clone --depth 1 -b <session-branch> https://github.com/Archont561/pixi-release.git /tmp/work
     cd /tmp/work && sh scripts/restore.sh            # ~1 min, ends with "restore complete"
     export PATH="$HOME/.local/bin:$PATH"             # pixi and pixi-sandbox land here
     pixi install --frozen --offline                  # must be a no-op
     pixi run --frozen gates                          # the baseline, offline
     ```

     `PIXI_SANDBOX_USER_TOOLS=skip` leaves `HOME` and the shell profile untouched (CI, shared
     accounts); `PIXI_SANDBOX_BRANCH` overrides the branch it resolves from `pixi-sandbox.toml`.
     The script is generated — regenerate it through `pixi-sandbox upgrade`, never edit it, and
     keep its `# pixi-sandbox-version:` stamp as it came.

     When the script cannot run (no GitHub reachability to fetch `sandbox/*`), fall back to the
     manual recipe in `backlog/docs/research/agent-environment/doc-7`. Either way, do this before
     planning, so the gates you promise are the gates that will actually run.
   - Backlog stays usable throughout through `npx -y backlog.md@1.53.0 …`, the exact version
     pinned in `package.json`. Only when a restore is impossible, treat CI as the authority:
     `gh run list --branch <branch> --limit 5`.
4. **GitHub.** `gh auth status`. If it fails with an authentication error, tell the user the GitHub
   connection needs attention and ask them to reconnect GitHub in Arena. Do not ask for tokens.
5. **Environment facts.** Read `backlog/docs/research/agent-environment/doc-7` before planning any
   release probe. It lists what the sandbox cannot reach (Actions logs, release asset uploads,
   conda-forge) and the CLI gotchas.

Record the outcome in the standup (template §2): what is up, what is down, and what that rules out.

## 2. Survey the Backlog and pick the sprint slice

Use the Backlog CLI. Do not hand-edit task Markdown (`AGENTS.md`).

```bash
B="npx -y backlog.md@1.53.0"      # or: pixi run backlog --   when pixi is present
$B milestone list                 # which milestone is next, and how far along it is
$B task list --json               # filter on .milestone to see the candidate slice
$B task view REL-N                # Dependency Graph, criteria, description
$B board
$B doc view doc-10                # the current sprint plan, if one exists
$B search "<keyword>"             # before proposing anything, check for duplicates
$B doctor                         # duplicate ids, dependency cycles
```

Pick the slice with the checklist `doc-10` §2 gives: milestone exists, every task carries its id,
every dependency finished (or inside the slice), base green, no claim contradicts a decision, every
task has a proof, the exit condition is written, the release path is decided. A task whose
dependency sits in a later milestone is **not** in this sprint — it is the next one's first row.

Read the decisions and documents a candidate task cites before proposing it. Decisions are the
rules: `backlog/decisions/`. Specifications are normative: `backlog/docs/`. Design history and
rejected directions: `doc-6`.

## 3. Propose the sprint, then stop

Write the proposal in the standup (template §2) and **stop for the user's go-ahead**. The proposal
states:

- the sprint's milestone and its ordered task list, with ids and the criteria you expect to prove
  for each;
- the sprint's exit condition, including the version you expect to cut and how it will be cut;
- anything you will *not* attempt, and why (environment, dependency, scope, decision gap);
- the check you will run per task (`pixi run gates` locally when available; CI otherwise);
- if the sprint's release path is blocked by an accepted decision, name the decision and the two
  honest options — do not assume the user wants the decision superseded.

Do not start implementation before the user confirms.

## 4. Work the sprint

Work the tasks in plan order. Per task, the loop is short:

1. **Claim it.** `backlog task edit <id> -s "In Progress"`.
2. **Prove it.** Failing test first, then the change; run the task's own proof command.
3. **Land it.** Conventional commit, push the session branch, open the pull request, watch checks,
   merge with `gh pr merge <n> --merge` (§5.2).
4. **Record it.** Append the hand-off (template §3) through the CLI, then move to the next task.

House rules that most often get broken:

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
- **Task-long detour.** If a task turns out to need many workflow runs or a long external wait, it
  is legitimate to finish the session task-long instead of the sprint — say so, confirm, and carry
  the rest of the sprint forward in the report.

## 5. Close the sprint

1. **Merge.** One pull request per task where practical; more than one commit is fine, more than one
   unrelated task per pull request is not. Normal merge commit only: `gh pr merge <number> --merge`,
   never `--squash`, `--rebase` or `--auto`. Name the number explicitly; the merge command must be
   the last command in its chain.
2. **Watch the checks.** `gh pr checks <number> --watch` or `gh run list --branch <branch>`. When a
   job fails, read its log before changing code: `gh run view <id>`; for the job log use
   `gh api repos/Archont561/pixi-release/actions/jobs/<job-id>/logs` (`gh run view --log` often
   cannot stream from this sandbox). Filter on a changed run id — `gh run list --limit 1` right after
   a push can return the previous run.
3. **Close the tasks.** Every acceptance criterion verified; status moved with
   `backlog task edit <id> -s Done`; notes appended through the CLI. **An AC you could not prove is
   not a checked AC.** Leave that task `In Progress`, name the missing proof in its notes, and carry
   the same sentence into the report.
4. **Check the milestone.** `backlog milestone list`. If the sprint was the milestone's last slice,
   the milestone is closed by evidence — every task `Done` with a green run — not by a checkbox.
5. **Decide the release.** A sprint ships one version (`decision-6`, `decision-7`).
   ```bash
   pixi release plan --out plan.json
   pixi release prepare --plan plan.json --dry-run
   pixi release cut --plan plan.json            # add --push to publish the tag
   ```
   - The tag is the evidence anchor and `cut` writes it (`decision-4`); a merge does not.
   - If the sprint should trigger a release automatically, the trigger must already be a recorded
     rule — `REL-16` owns the milestone-tag question, and `decision-7` forbids a second tag per
     version. **Never invent a second tag authority to make a release run.**
   - A refused cut is a result: record the refusal key in the report instead of working around it.
6. **Re-baseline.** `git fetch origin && git log --oneline origin/main -1`, then gates on the merged
   tree. Put the result in the report.
7. **Record what you did not implement.** Proposals weighed and rejected, measurements worth keeping,
   and anything deferred go into Backlog — as a task (`backlog task create`) or into `doc-6` as a
   rejected direction. Do not leave them only in the chat, and do not add a new file to the
   repository for them.
8. **Report** (template §4), then **write the next sprint plan and its opening prompt**: update or
   create the `docs/plan` document for the next slice (`doc-10` is the shape), and fill the opening
   prompt so a fresh session reaches the same understanding without reading the diff.

A sprint that merged nothing still reports: "Merged: nothing", what was recorded, which stop
condition fired, and an opening prompt that says what to decide first.
