---
id: REL-32
title: Retire the repair input once publish can resume
status: To Do
assignee: []
created_date: '2026-10-24 18:12'
updated_date: '2026-10-24 18:12'
labels:
  - release
  - ci
milestone: m-2
dependencies:
  - REL-6
documentation:
  - >-
    backlog/docs/plan/doc-11 -
    Development-plan-version-truth-evidence-then-surface.md
priority: medium
type: chore
ordinal: 32000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
release.yml grew an existing_run_id input on 2026-10-24 to finish v0.1.0 from the artifacts that run had already produced (REL-27, PR #44). It works, and it is the right emergency patch, but it must not become the permanent interface: it is a second way to publish, expressed in YAML, and AGENTS.md allows one command path. Once REL-6 gives publish a per-target evidence record and a resume mode, the workflow input is deleted and the repair becomes a command a consumer can run too.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A repair run is expressed as a publish resume against the recorded evidence, with no workflow-specific input naming a previous run id.
- [ ] #2 existing_run_id is removed from release.yml, and a re-run of the v0.1.0 shape (artifacts produced, last target failed) completes through the documented command alone.
- [ ] #3 The rendered lane carries the same single path: no rendered workflow offers a second way to publish.
- [ ] #4 REL-27 notes are updated to point at the command that replaced the input, so the workaround is not left as the documented remedy.
- [ ] #5 Proven by a green gates run on the merge commit, and by one recorded run that resumed a deliberately failed target.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 The deletion is mechanical: no compatibility shim keeps the input alive under a new name.
<!-- DOD:END -->
