# AGENTS.md — pixi-release operating rules

## Mission and current boundary

`pixi-release` is a typed release controller and GitHub workflow renderer for Pixi projects. It owns release orchestration, evidence, verification, resume semantics, and generated workflow checks. It does **not** become a second SemVer/changelog authority: when the project is configured for Convco, Convco remains authoritative.

This is a scaffold. The dummy crates must remain honest: no command may claim to publish, tag, or verify a release until its adapter and evidence model exist.

## Canonical project system

Backlog.md is the canonical operating system for delivery work and approved project knowledge:

- Create and update tasks, subtasks, milestones, dependencies, plans, acceptance criteria, definition of done, project docs, and decisions through `pixi run backlog -- …`; do **not** hand-edit task Markdown.
- Use the Backlog search and board surfaces before planning duplicate work.
- Store approved/normative architecture in Backlog decisions and documents, linked to implementation tasks.
- **One canonical source.** A statement exists in exactly one place: its rule in a decision, its specification in a document, its measurement in the research or spike document that produced it. Cite it by Backlog id and section — never restate it, and never leave a second copy behind.
- Promote a conclusion into Backlog.md before treating it as a project rule. The test: **if someone would be wrong to contradict it, it is a Backlog document or decision.**
- There is no `.knowledge/` directory. Design history, rejected directions, and working-environment facts are Backlog documents: `doc-6` (design history and index of determinations) and `doc-7` (agent sandbox environment). Do not recreate `.knowledge/`.
- Stable anchors: section numbers (`doc-2` §2.6) and decision ids (`decision-4`) may be cited from code and commits. Renumber nothing without fixing the citations.
- Tag every claim in a research, spike or environment file ✅ measured or ⚠️ reasoned.

The configured task prefix is `REL`.

## One command surface

Never add a second CI or hook command path. The route is:

```text
pixi run <verb> → root package script → bun x turbo run <verb> → package script
```

- `pixi.toml` owns toolchain versions and repo-global commands.
- `package.json` owns the Bun workspace and root Turbo scripts.
- `crates/package.json` owns Cargo's Turbo façade, covering every member including `xtask`.
- `turbo.json` owns ordering/cache policy.
- `lefthook.yml` and `.github/workflows/ci.yml` call only `pixi run <verb>`.

Run `pixi run gates` before handing off a change. Scope Turbo work only by forwarding an explicit filter, for example `pixi run test -- --filter=@pixi-release/rust`.

## Rust boundaries

- Put release-domain data and deterministic pure behavior in `pixi-release-core`.
- Put Git semantics behind `pixi-release-git::GitProtocol`; test policy with `FakeGit` before adding shell integration.
- Keep `pixi-release` thin: parsing and presentation belong there, domain rules do not.
- New crates opt into `[lints] workspace = true`; do not restate lint policy.
- Rust tests use `rstest` fixtures, `proptest` properties and `rstest-bdd` scenarios, written test-first per the `tdd` skill; dummy repositories come from a shared builder (`decision-11`).
- Tests must prefer behavior and public APIs. External process and real-local-bare-repository tests come after the fake proves policy behavior.

## Session workflow

Start and end every work session with `.agents/skills/session/SKILL.md`. It sequences environment bring-up, backlog survey, proposal, work, merge, and hand-off.

## Merge policy

- Every pull request merges with a normal merge commit: `gh pr merge <number> --merge`. Never `--squash`, `--rebase`, or `--auto`.
- Do not rewrite history on `main` or on a pull request branch after review.
- Do not push to any branch other than the session branch.

## Release safety rules

- Authored policy: `.pixi-release.toml` (doc-3 §4, not yet present in this repository); generated workflows: YAML; immutable plans/manifests: JSON.
- The future lifecycle is `plan → prepare → cut/tag → verify/build → publish → resume`.
- `cut` must never publish by default. Publish must validate evidence and be resumable per target.
- Self-release must install a pinned, verified prior release—not execute the unreleased checkout as its own controller.
- Do not add a registry target until its artifact manifest and resume behavior are specified and tested.

## What not to commit

Do not commit `.pixi/`, `node_modules/`, `.turbo/`, `target/`, or `.pixi-release/`. Lockfiles are committed and reviewed. `.gitignore` enforces this.
