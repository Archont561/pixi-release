---
id: REL-19
title: Decide the public documentation site contract
status: To Do
assignee: []
created_date: '2026-10-23 15:01'
labels:
  - architecture
  - release
milestone: m-4
dependencies:
  - REL-11
references:
  - 'https://github.com/Archont561/pixi-release/issues/10'
priority: medium
type: spike
ordinal: 19000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Issue #10 asks for `pixi release docs init`, `sync`, `api` and `workflow`, an Astro Starlight scaffold and a generated GitHub Pages workflow. doc-3 §1.4 removed `apps/docs/` and left no public docs home; decision-10 §2 keeps `docs.yml` hand-written in the consumer; doc-8 specifies only a presentation sidecar that reads a release manifest. This task decides where docs live, whether pixi-release scaffolds them at all, and which release data they may read.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A decision records whether documentation scaffolding is in scope for pixi-release, and confirms or amends the renderer scope of decision-10 §2.
- [ ] #2 Where the site lives and who owns `docs.yml` are recorded, superseding or confirming doc-3 §1.4.
- [ ] #3 The release data contract is fixed: which manifest fields from REL-11, which Convco changelog artefact (decision-6) and which digests a site may read, with no second copy of release facts.
- [ ] #4 The relationship to the doc-8 presentation sidecar is recorded: consume it, embed it, or neither.
- [ ] #5 No scaffold or generator is merged under this task.
<!-- AC:END -->
