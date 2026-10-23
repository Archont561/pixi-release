---
id: decision-13
title: clap owns the pixi-release command grammar
date: '2026-10-23 21:51'
status: accepted
---
## Context

`crates/pixi-release` parsed `argv` by hand in four places: a top-level `match` on the shape of
the argument list, plus a `while let` loop for each of `plan`, `prepare` and `cut`, all sharing one
`USAGE` string. That was honest at two subcommands and ten flags. `verify`, `publish` and `resume`
each bring their own flag set, and every one of them would have added a fourth hand-written
parser whose usage text and exit codes drift from the others.

The sibling tool this product is built against — `pixi-sandbox` — is itself a `clap` CLI, so the
owner's ecosystem already has one answer to "what does a `pixi` subcommand look like".

## Decision

`clap` derives the grammar. `#[derive(Parser)]` on a `Cli` type declares the subcommands, every
flag, and the help text; `Cli::parse()` owns `argv`, usage errors, `--help` and `--version`.

`clap` is pinned at `4.6.7` with `default-features = false` and exactly these features: `std`,
`help`, `usage`, `error-context`, `suggestions`, `derive`. Color is deliberately off: it would
pull `anstream` and five more crates for output that CI strips anyway, and keeping it off means
the assertions in `tests/cli.rs` never see an escape sequence.

## Cost

Six crates enter the lock — `clap`, `clap_builder`, `clap_lex`, `clap_derive`, `anstyle` and
`strsim` — on top of a tree that already had `syn 3`, `proc-macro2`, `quote`, `unicode-ident` and
`heck`. `anstyle` is not optional in `clap_builder 4.6`; `strsim` arrives with `suggestions`.
`pixi-release` keeps `publish = false`, so none of this reaches a consumer's build.

## Invariants the migration had to keep

- `pixi-release --version` still prints exactly `pixi-release <CARGO_PKG_VERSION>`.
- A usage error still exits `2`, a refusal still exits `1`, success still exits `0`. That split is
  what the CLI tests assert, so it is contract, not convenience.
- `plan --version <V>` remains the *explicit version* flag. The top-level version flag is
  therefore not propagated into subcommands (`propagate_version` stays off), or `plan --version`
  would become ambiguous.
- `prepare` still refuses to run without `--dry-run`, now as a required flag rather than a
  hand-written check — same message, same exit code.
- No command gains a side effect. The grammar change is a change of parser, not of behaviour: all
  fourteen pre-existing CLI tests passed unmodified before the four grammar tests were added.

## Consequences

Usage text is generated, so the CLI tests assert on observable behaviour (`--help` lists the
commands, an unknown flag exits `2`, an unknown bump value is named) rather than on a `USAGE`
constant. If a consumer's own CLI wants `clap`, that is their decision; nothing here is rendered
into consumer repositories (`decision-12`).
