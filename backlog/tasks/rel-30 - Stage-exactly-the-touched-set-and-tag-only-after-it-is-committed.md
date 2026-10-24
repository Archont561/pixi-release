---
id: REL-30
title: Stage exactly the touched set and tag only after it is committed
status: In Progress
assignee: []
created_date: '2026-10-24 18:11'
updated_date: '2026-10-24 19:24'
labels:
  - release
  - testing
milestone: m-1
dependencies:
  - REL-4
documentation:
  - >-
    backlog/docs/plan/doc-11 -
    Development-plan-version-truth-evidence-then-surface.md
priority: high
type: feature
ordinal: 30000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
A publish job must consume a tree that already agrees with its tag. Today release.yml checks out an already-pushed tag, stamps the version into the worktree and never commits it, so the artifact is built from a tree the tag does not describe; the stamp step was also a no-op on v0.1.0, which is why REL-27 AC#2 cannot be proved. Split the write from the cut: prepare writes and reports, commit stages exactly the reported set and refuses an unaccounted modification, cut tags the commit it just made. Then delete the stamp step from release.yml. Modelled on pixi-sandbox's xtask commit-release, which is fixture-tested against a real bare remote.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 commit stages exactly the paths prepare reported and nothing else; an unaccounted modification anywhere in the worktree refuses the commit, naming the path.
- [ ] #2 cut tags the commit commit created, and refuses when HEAD is not that commit or the tag already exists on the remote.
- [x] #3 release.yml no longer stamps a version: the build job verifies that the tagged tree already agrees with the tag and fails with the disagreeing path when it does not.
- [ ] #4 REL-27 AC#2 becomes provable: a run whose tag names a version the manifests do not carry fails at the check, and a run whose tag matches passes without editing anything.
- [x] #5 Proven by a green gates run on the merge commit, including a bare-remote fixture that exercises push, tag and the refusals.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 The stamp-after-tag path is gone from every rendered and hand-written workflow, not just from the owner lane.
- [x] #2 The refusal messages name the remedy (run prepare, or remove the stray change), not only the fault.
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented on the session branch. AC#1/#2 were already satisfied by `cut` (REL-4): `cut::check_repository` refuses a dirty path outside `plan.touched()`, `cut::cut` stages exactly `plan.touched()`, commits, then tags the commit it made; proven by `cut_commits_and_tags_exactly_the_recorded_set`, `cut_refuses_a_stale_plan` and the new `a_change_outside_the_recorded_set_is_refused_by_path_and_remedy` (the refusal now names the path *and* the remedy: run prepare again, or revert it). AC#2's remote-tag half is local-tag only: `check_repository` refuses a tag that already exists in the repository, and a tag that exists only on the remote still surfaces at push, after the local commit and tag exist — recorded here rather than claimed.

AC#3/#4: the stamp step is deleted from `release.yml`; the build job now runs `xtask check --tag <tag>`, which compares the three manifest literals and the recipe's `context.version` to the tag and emits one ::error:: per disagreeing path, naming the remedy. Measured on main + this diff, both directions of the exact command the step runs: `--tag v0.1.0` → exit 0, `xtask check: repository checks passed (tag v0.1.0 describes the tree)`; `--tag v9.9.9` → exit 1, four findings (Cargo.toml, pixi.toml, package.json, recipe), `xtask check: 4 finding(s)`, worktree byte-identical either way. The negative case is the one that had never fired; it is now a unit test (`a_tag_that_names_another_version_is_refused_naming_the_path`) as well as a measured run. AC#4's CI half is not proven: a workflow-level negative run needs a tag that disagrees with its tree pushed into the repository, which spends version space, and `gh workflow run` is 403 for this session's token — so AC#4 stays open until a release run passes the verify step and an owner starts the negative one.

Proof: `pixi run --frozen gates` exit 0 (fmt, clippy -D warnings, workspace tests including 12 xtask tests, actionlint, workflow check, repo-check). AC#5 pending the merge-commit run.

DoD#1 measured: `grep -rn stamp .github/workflows pixi.toml crates --include=*.{yml,yaml,toml,mjs,ts}` finds only the comment explaining what the deleted step did. There are five hand-written workflows (autorelease, ci, publish-sandbox, release, relock) and no rendered one yet — the renderer is REL-5 — so the owner lane was the only place the pattern existed.

Merge proof: PR #51 → merge commit `22e0dc8`; branch run `40124721058` success (gates 2m46s, package 40s), `main` push run `40124947471` success, `autorelease` `40124947470` success (observe path, no tag). AC#5 checked — the `pixi-release-git` suite with its bare-remote fixture runs inside `gates`. REL-30 stays In Progress: AC#2's remote-tag half and AC#4's workflow-level negative run are open and recorded above.
<!-- SECTION:NOTES:END -->
