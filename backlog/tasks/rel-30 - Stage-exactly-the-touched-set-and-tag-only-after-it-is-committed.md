---
id: REL-30
title: Stage exactly the touched set and tag only after it is committed
status: To Do
assignee: []
created_date: '2026-10-24 18:11'
updated_date: '2026-10-24 18:12'
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
- [ ] #1 commit stages exactly the paths prepare reported and nothing else; an unaccounted modification anywhere in the worktree refuses the commit, naming the path.
- [ ] #2 cut tags the commit commit created, and refuses when HEAD is not that commit or the tag already exists on the remote.
- [ ] #3 release.yml no longer stamps a version: the build job verifies that the tagged tree already agrees with the tag and fails with the disagreeing path when it does not.
- [ ] #4 REL-27 AC#2 becomes provable: a run whose tag names a version the manifests do not carry fails at the check, and a run whose tag matches passes without editing anything.
- [ ] #5 Proven by a green gates run on the merge commit, including a bare-remote fixture that exercises push, tag and the refusals.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 The stamp-after-tag path is gone from every rendered and hand-written workflow, not just from the owner lane.
- [ ] #2 The refusal messages name the remedy (run prepare, or remove the stray change), not only the fault.
<!-- DOD:END -->
