---
id: decision-15
title: The name is load-bearing - pixi-release keeps its prefix
date: '2026-10-24 17:08'
status: accepted
---
## Context

The naming question was asked once, at the point where the answer becomes expensive: `v0.1.0` is
the first version to publish, and after it lands in `archont561/archont561` the name is a
migration, not an edit. Three plausible names existed — keep `pixi-release`, rename to an
independent brand with a `pixi-<brand>` shim, or drop the prefix — and the argument for each was
aesthetic until measured.

The measurement is that **the prefix is a mechanism, not decoration**. Pixi discovers a
subcommand by looking for an executable called `pixi-<command>` on `PATH` and in its global
directories, so `pixi release plan` works only because the binary is `pixi-release`. The same
string is the install identity: the conda package name, the channel artifact, the recipe's
package name, and the `pixi global install … pixi-release` contract (`doc-2` §1.1). Renaming it
silently removes the subcommand and replaces the installable.

## Decision

1. **The binary stays `pixi-release`.** The `pixi-` prefix is the extension-discovery key and
   the install identity in one string; nothing in this repository may rename it without a
   migration that keeps both working. `pixi release <verb>` is the user-facing spelling, and
   `pixi-release <verb>` is the same command under its real name.
2. **The conda package name stays `pixi-release`.** It is what `archont561/archont561`
   publishes and what `pixi global install --channel … pixi-release` resolves. A rename later is
   a second package plus a deprecation stub, never an edit of the published one
   (`decision-7`: one version, one tag, one artifact).
3. **The prefix is descriptive, not borrowed.** `decision-2` fixes the split — controller work
   is `pixi-release <verb>`, repository work is `pixi run <task>` — and `crates/pixi-release/src/gates.rs`
   executes gates through `pixi run` rather than inventing a second command path. The policy is
   `.pixi-release.toml`. There are no `PIXI_RELEASE_*` environment variables, so the name is not
   a hidden surface either.
4. **`pixi release` is a convenience we do not own.** Pixi already ships the adjacent verbs
   (`pixi publish`, `pixi upload`); if prefix.dev later adds a built-in `release`, the built-in
   wins and the tool remains `pixi-release <verb>`. No design may depend on owning the
   subcommand name — which is also why the workflows invoke the binary, never `pixi release`.
5. **Branding may change; the mechanism may not.** A product name, a tagline or a README voice is
   free to evolve. The two invariants above are not branding.

## Consequences

- `v0.1.0` publishes as `pixi-release`, so the decision is taken before it costs anything — the
  reason it is recorded now rather than when someone first asks.
- A future rename proposal must cite this decision and carry the migration: an alias binary plus
  a deprecation package, with the old name still resolving for at least one minor version.
- Renaming the **repository** is a separate question, not covered here: it changes the
  trusted-publishing binding — repository, workflow file and `prefix` environment
  (`decision-14` §6) — and would need the publisher re-registered.
