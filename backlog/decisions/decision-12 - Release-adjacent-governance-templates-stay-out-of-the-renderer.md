---
id: decision-12
title: Release-adjacent governance templates stay out of the renderer
date: '2026-10-23 19:43'
status: accepted
---
## Context

Issue #9 asks for three governance artifacts — issue forms with a scope dropdown, an issue
`config.yml`, a PR template, and a PR-title check. `decision-10` §2 confines the renderer to the
release lane, and `doc-3` §2.2 names the only two files it renders, so admitting any of them needs
a decision rather than a generator.

Three facts decide the shape:

- ✅ **A GitHub issue form is static YAML.** A `dropdown` carries its `options` list inside the
  file, and a form cannot read `.versionrc` at render or at submit time. Any populated scope
  dropdown is therefore a generated copy that must be re-generated on every change.
- ✅ **Convco enumerates no scopes.** `.versionrc` exposes `scopeRegex` (default
  `[[:alnum:]]+(?:[-_/][[:alnum:]]+)*`), which `convco check` applies to a commit message; the
  accepted scope set is whatever the regex matches. "Align the dropdown 1:1 with the `.versionrc`
  scopes" has no referent: there is no list to align with.
- ✅ **PR titles are not release input here.** `AGENTS.md` mandates normal merge commits, and
  Convco ignores merge commits by default, so a PR title never reaches the changelog. What
  Convco does read is the commits inside the branch — and `convco check --from-stdin` already
  validates a single message against the same `.versionrc`.

## Decision

Each proposed artifact is judged on its own, and each names its lane as `decision-10` §2 defines
it: the **release lane** is rendered by `pixi-release`; every other lane is hand-written by the
owner or the consumer.

1. **Issue forms, `.github/ISSUE_TEMPLATE/*.yml` — rejected.** Governance lane. A populated scope
   dropdown can only be a second enumeration that drifts from `scopeRegex`, which `decision-6`
   puts out of bounds; and no issue form is read when a version or a changelog entry is derived,
   so its absence costs the release record nothing.
2. **Issue `config.yml` — rejected.** Governance lane. Disabling blank issues is an editorial
   choice about the tracker, not a release fact.
3. **PR template, `.github/PULL_REQUEST_TEMPLATE.md` — rejected.** Governance lane. It is prose
   that no tool reads, so it cannot be drift-gated and will rot silently.
4. **PR-title check — rejected as a rendered file and as a new subcommand; admitted as one
   hand-written line in the consumer's own CI.** Convco already answers the question:
   `printf '%s\n' "<title>" | convco check --from-stdin` reads the same `.versionrc`, so the scope
   list stays single-sourced without any generation. Shipping `types` or `scopes` lists to a
   third-party action would re-create the enumeration this decision refuses. The line belongs in
   the consumer's `ci.yml`, which `decision-10` §2 keeps hand-written.

Nothing is rendered, so `decision-3`'s marker, stamp, policy digest and drift gate have no new
file to cover — waived explicitly, not forgotten. The waiver lapses the moment any of these
becomes a rendered artifact, and no rendered file may carry a scope enumeration.

## Consequences

- `decision-10` is **confirmed, not amended**: the renderer still claims `release.yml` and
  `auto-release.yml` only. `doc-3` §2.2 and `REL-5` keep their file list unchanged.
- `workflows.render` in `.pixi-release.toml` stays `auto-release`/`release`, and the parser's
  admissible set does not widen (`REL-2`).
- `REL-18` is withdrawn: its generated-file acceptance criteria are superseded. `doc-6` records
  the rejection by id.
- The one-liner is documented for consumers in `doc-2` §2.7, which is the only place it needs to
  appear.
- **Reopen condition.** If a consumer adopts squash merging, the PR title becomes release input.
  Reopen this decision then; the first admissible shape remains `convco check --from-stdin` in the
  consumer's own CI, never a generated enumeration.
