---
id: REL-34
title: >-
  Detect the version from known manifests, with declared markers as the escape
  hatch
status: To Do
assignee: []
created_date: '2026-10-24 18:25'
labels:
  - release
milestone: m-2
dependencies:
  - REL-11
priority: high
type: feature
ordinal: 34000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Reading a version should not require the caller to know which line carries it. For the manifests every ecosystem already agrees on — Cargo.toml (workspace or package), package.json, pyproject.toml, pom.xml, *.csproj and Directory.Build.props, go.mod, and a conda recipe.yaml — the format, key and scheme are knowable from the file, so pixi-release resolves them and REFUSES anything it cannot classify rather than scanning for a version-shaped line. Two escape hatches cover the rest: an in-file marker naming the line that carries the version, and an explicitly declared entry in policy for a file no table recognises. Whatever is resolved is written into the plan's evidence (path, format, key, scheme, detected-or-declared), so nothing is guessed silently at cut time. The version source stays Convco (decision-6); this decides how a literal is READ, not which version is next. Open, and to be settled before this starts: whether an environment variable may declare a version (as a manifest that must agree) or only name one to be stamped, and what that variable is called — decision-15 records that this tool exposes no PIXI_RELEASE_* variables, so introducing one is a decision change, not a detail.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Each manifest listed by a bare path is resolved from a closed table of known files; the resolved format, key and scheme appear in the plan evidence.
- [ ] #2 A file the table does not recognise is refused with the explicit declaration it needs, never read by heuristic; the refusal names the path and the accepted formats.
- [ ] #3 An in-file marker names the line carrying the version, and the same marker is honoured by both the checker and the writer so they cannot disagree about which line is the version.
- [ ] #4 An explicit policy entry overrides detection for one path, and a test proves detection and declaration produce the same evidence when they agree.
- [ ] #5 Detection is deterministic: the same tree yields the same evidence twice, and a fixture pins the whole resolved table.
- [ ] #6 Proven by a green gates run on the merge commit covering these tests (decision-11 fixtures).
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 The known-file table is data with a test per entry, not a match inside a reader.
- [ ] #2 Every refusal names the remedy: the marker syntax or the policy entry that would make the file readable.
<!-- DOD:END -->
