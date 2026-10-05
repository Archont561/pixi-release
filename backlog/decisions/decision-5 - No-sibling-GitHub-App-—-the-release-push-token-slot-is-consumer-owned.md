---
id: decision-5
title: No sibling GitHub App — the release push-token slot is consumer-owned
date: '2026-10-04 21:06'
status: proposed
---
## Context

A sibling GitHub App was proposed to carry release behaviour. The attraction is real and
threefold: an app installation token's pushes and tags start workflows — dissolving the
dispatch chain that exists because *"a `GITHUB_TOKEN` push or tag starts no workflow"*
(`doc-4` §6.1); an app token passes branch protection cleanly where `GITHUB_TOKEN` cannot; and
a PR opened with an app token receives a CI verdict, where the relock incident (`doc-4` §6.1)
and `release-upgrade.yml` both get none.

Measured against that (✅ 2026-10-04): **no `Archont561/*` default branch carries rulesets**
(0 on all six public repositories), and this repository is private on a free plan, where
branch protection is not even available. The strongest pro-app pressure has zero current
instances in the consumer population.

Two variants were considered. **(B) A hosted webhook bot** that performs release behaviour
server-side fails three standing decisions at once: it is a deployed service with
availability and state where `decision-2` forbids even a compiler; it breaks `decision-4`'s
model of `cut` as a CLI acting on a worktree (no offline `rehearse`, fakes no longer cover
production); and it is a second version/deployment domain — the same defect that rejected the
composite `setup/action.yml`. **(A) A token-minting app owned by `pixi-release`** trades the
family's no-standing-secrets stance (OIDC publish, `doc-4` §5a) for a long-lived private key
whose blast radius is every installed repository, and makes adoption (`doc-2` §1.1) depend on
installing someone else's app — untenable for any consumer outside `Archont561/*`.

## Decision

> `pixi-release` ships no GitHub App, in either form. Workflow causality stays explicit —
> the renderer-emitted, drift-gated dispatch chain — and the privileged-token slot accepts a
> **consumer-owned** credential without caring how it was minted.

1. **The dispatch chain is the default and the product.** Hand-offs in pixi-sandbox were
   fragile because they were hand-written; here they are rendered one-liners, covered by
   `workflow check`, executed by tested subcommands. Implicit app-token event chains move
   causality back into forge event semantics, which `doc-2` §2.5 already rejects as
   untestable against a fake forge.
2. **`RELEASE_PUSH_TOKEN` is the one slot, and it is credential-agnostic.** Any push-capable
   token is admissible: a fine-grained PAT, or an installation token from an app the
   *consumer* registers and owns. The renderer may optionally emit a SHA-pinned
   `actions/create-github-app-token` step to mint it in-run (pin: `doc-4` §7b). `pixi-release`
   never owns, registers, or requires an app.
3. **When the token is present, it wins.** Pushes, tags and PRs made with it trigger
   workflows naturally; rendered dispatch steps that exist only to compensate for
   `GITHUB_TOKEN` inertness are skipped, not duplicated.
4. **The upgrade PR must get a verdict.** By default — no secrets configured —
   `pixi-release upgrade pr` dispatches `ci.yml` against the PR head and attaches the result,
   so the relock incident cannot replay. With the token configured, CI triggers naturally and
   the dispatch is skipped (rule 3).

## Consequences

- Adoption stays at `doc-2` §1.1's four minutes with zero secrets; consumers behind strict
  branch protection have a documented path that costs them one secret and no new trust in
  this project.
- The family's credential posture is preserved: the only long-lived keys anywhere are ones a
  consumer chose to create, scoped to their own repositories.
- `upgrade pr` grows two testable behaviours (dispatch-verdict default, token-natural
  alternative) instead of one untested hole; `release-upgrade.yml` needs `actions: write`.
- Revisit trigger, recorded honestly: if a measured consumer population adopts rulesets that
  block `GITHUB_TOKEN` *and* the one-secret path proves a real adoption barrier, the
  token-minting variant (A) may be reopened. Variant (B) is closed on architectural grounds,
  not circumstances.
