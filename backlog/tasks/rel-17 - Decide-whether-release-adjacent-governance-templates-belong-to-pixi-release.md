---
id: REL-17
title: Decide whether release-adjacent governance templates belong to pixi-release
status: Done
assignee: []
created_date: '2026-10-23 15:01'
updated_date: '2026-10-23 19:44'
labels:
  - architecture
  - ci
milestone: m-4
dependencies: []
references:
  - 'https://github.com/Archont561/pixi-release/issues/9'
priority: medium
type: spike
ordinal: 17000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Issue #9 asks for generated issue forms with workspace scopes, a PR template, and a semantic PR-title workflow. decision-10 §2 confines the renderer to the release lane and doc-3 §2.2 owns only `release.yml` and `auto-release.yml`, so admitting these needs a decision, not a generator. The decision also fixes the single scope source: the Convco configuration, because decision-6 makes it the only version and changelog authority.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A decision records each proposed artifact (issue forms, issue `config.yml`, PR template, PR-title check) as admitted or rejected, with the reason and the lane it belongs to.
- [x] #2 The scope list is single-sourced from `.versionrc` and Convco, with no second enumeration of scopes in any generated file (decision-6).
- [x] #3 The decision-3 rendering rules (marker, stamp, policy digest, drift gate) are applied to each admitted file or waived explicitly.
- [x] #4 decision-10 is confirmed or amended by the recorded decision, whichever the outcome requires, in the same change.
- [x] #5 No generator code is merged under this task.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Answered by `decision-12` (accepted 2026-10-23), with consumer-facing detail in `doc-2` §2.7 and the withdrawal recorded in `doc-6` §Rejected directions.

Per artifact: issue forms, issue `config.yml` and the PR template are rejected — governance lane, and a scope dropdown can only be a generated copy because GitHub issue forms are static YAML while Convco offers `scopeRegex` rather than a scope list. The PR-title check is rejected as a rendered file and as a subcommand, and admitted as one hand-written `convco check --from-stdin` line in the consumer's `ci.yml`.

AC#2: the only scope source named anywhere is `.versionrc`; no artifact carries a list, because nothing is generated. AC#3: no file is rendered, so decision-3's marker/stamp/digest/drift gate is waived explicitly, with the waiver lapsing on any future rendered governance file. AC#4: `decision-10` is confirmed rather than amended — the renderer keeps `release.yml` and `auto-release.yml` only — and `.pixi-release.toml`'s `workflows.render` admissible set is unchanged. AC#5: no generator code was added; the change is Backlog documents plus the withdrawal of REL-18.
<!-- SECTION:NOTES:END -->
