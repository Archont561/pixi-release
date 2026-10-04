# pixi-release

`pixi-release` is a Rust-powered [Pixi extension](https://pixi.prefix.dev/latest/integration/extensions/introduction/) for planning, preparing, verifying, and publishing a safe release. Once installed as `pixi-release`, Pixi discovers it as:

```console
pixi release plan
```

> **Scaffold status:** this repository intentionally contains a compiling three-crate Rust skeleton and a working cross-language task graph. The lifecycle commands currently describe their intended phase only; release mutation and publication are not implemented yet.

## One command surface

Pixi is the entry point. Package-local commands live in `crates/package.json` and `apps/docs/package.json`; Turbo fans them out; Pixi owns the reproducible toolchain and exposes the common verbs:

```console
pixi run setup       # install the Bun workspace and Git hooks
pixi run gates       # lint, type-check, test, actionlint, workflow check, repo checks
pixi run build       # Turbo builds the Rust workspace and documentation stub
pixi run fmt         # apply all formatters
pixi run backlog -- board
```

`pixi run lint`, `pixi run typecheck`, `pixi run test`, `pixi run build`, and `pixi run fmt` each delegate to `bun x turbo run <verb>`. CI and Lefthook use those same Pixi verbs; neither reaches a package manager binary directly.

## Repository layout

- `crates/pixi-release-core` — deterministic release-domain types and planning invariants.
- `crates/pixi-release-git` — semantic Git port plus a stateful fake for unit tests.
- `crates/pixi-release` — the `pixi-release` executable; today it is a deliberately honest scaffold CLI.
- `crates/xtask` — owner-lane repository automation. Not the product: it is what cuts v0.1.0, because `pixi-release` may not be used to release `pixi-release`. It is deleted once a pinned prior release has been proven to do the job.
- `crates/package.json` — Turbo façade for Cargo, covering every member.
- `apps/docs/` — private Bun workspace package for public documentation.
- `templates/workflows/` — golden renderer templates for the consumer lane.
- `tests/fixtures/consumers/` — miniature consumer repositories in the three shapes the renderer must satisfy.
- `.pixi-release.toml` — authored release policy, reviewed as data. The tool never rewrites it.
- `backlog/` — canonical tasks, milestones, decisions, and project documents, managed only through Backlog.md.
- `.knowledge/` — pointers, design history, and working-environment notes. Canonical text lives in `backlog/`; start at `.knowledge/README.md`.

## Bootstrap

Install Pixi, then materialise the locked tools and hooks:

```console
pixi install --locked
pixi run setup
pixi run gates
```

The committed `pixi.lock`, `bun.lock`, and `Cargo.lock` are the three resolution records. Do not regenerate a lockfile incidentally; refresh it intentionally and review the diff.

## Delivery model

Authored policy belongs in [`.pixi-release.toml`](.pixi-release.toml). GitHub Actions YAML is generated/reviewed output; immutable plan and publication manifests are JSON. Convco remains the configured SemVer and changelog authority in Convco mode.

See `AGENTS.md` before changing project structure or delivery policy.
