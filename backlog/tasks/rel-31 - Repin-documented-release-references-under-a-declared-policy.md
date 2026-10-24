---
id: REL-31
title: Repin documented release references under a declared policy
status: To Do
assignee: []
created_date: '2026-10-24 18:12'
updated_date: '2026-10-24 18:12'
labels:
  - release
milestone: m-2
dependencies:
  - REL-28
documentation:
  - >-
    backlog/docs/plan/doc-11 -
    Development-plan-version-truth-evidence-then-surface.md
priority: medium
type: feature
ordinal: 31000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Markdown has no build step, so every literal release reference in it is a future lie: an install one-liner naming the previous tag sends a reader to the previous binaries. Declare the three classes in policy and honour each: stamped (README family, rewritten from the one predicate the checker also uses), derived (a literal is an error; the page must interpolate), historical (a changelog or a closed task record is supposed to name old releases) with a line-level opt-out for a walkthrough that must stay pinned. Only lines that name our own release qualify; a third-party pin is someone else's release and is never rewritten.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A policy entry declares stamped, derived and historical reference targets; a document outside all three is untouched.
- [ ] #2 prepare rewrites a stamped reference to the planned tag and reports the file in its touched set; verify reports the file and line when a stamped reference disagrees.
- [ ] #3 A literal release tag inside a derived target fails verify with the placeholder it should have used, instead of being rewritten.
- [ ] #4 An opted-out line keeps its tag through both prepare and verify; a line pinning a third party (another repository or action) is never rewritten.
- [ ] #5 Rewriting is byte-faithful: line endings and a missing trailing newline survive, proved by a fixture whose file has CRLF and no final newline.
- [ ] #6 Proven by a green gates run on the merge commit covering these tests (decision-11 fixtures).
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 The checker and the rewriter share one predicate, asserted by a test that a line either both report and both rewrite, or neither.
- [ ] #2 The opt-out marker is documented in the policy schema, not only in code.
<!-- DOD:END -->
