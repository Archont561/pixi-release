# pixi-release

<p align="center">
  <a href="https://github.com/Archont561/pixi-release/actions/workflows/ci.yml"><img src="https://github.com/Archont561/pixi-release/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://pixi.prefix.dev"><img src="https://img.shields.io/badge/Pixi-%E2%89%A50.81-44a833" alt="Pixi >=0.81"></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Rust-1.98-black?logo=rust" alt="Rust 1.98"></a>
  <a href="https://bun.sh"><img src="https://img.shields.io/badge/Bun-1.3-black?logo=bun" alt="Bun 1.3"></a>
  <a href="https://github.com/Archont561/pixi-release/blob/main/LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue" alt="MIT OR Apache-2.0"></a>
</p>

**A release controller for [Pixi](https://pixi.prefix.dev) projects.** `pixi-release` plans, prepares, verifies, and publishes a release as a sequence of resumable, evidence-backed steps, so a wrong policy is caught before anything is cut. Once installed as `pixi-release`, Pixi discovers it as a subcommand:

```console
pixi release plan
pixi release prepare --plan release-plan.json --dry-run
pixi release cut --plan release-plan.json
```

It decides *what* must be built, *where*, and *what must come back*. The repository decides *how*. The controller never runs a compiler. Design rules live in [`backlog/decisions/`](backlog/decisions/) and the specifications in [`backlog/docs/`](backlog/docs/).

> **Scaffold status.** The repository contains a working three-crate Rust workspace and a cross-language task graph. `plan`, `prepare --dry-run` and `cut` are implemented against a local Git adapter: `plan` derives canonical JSON, `prepare` reports the edits it would make without touching the tree, and `cut` refuses anything unsafe before it commits, tags, and optionally pushes. Publication, artifact verification, and changelog writing are not implemented yet, and no command claims to publish or verify a release.

## Quick start

Install Pixi, then materialise the locked toolchain, dependencies, and Git hooks:

```bash
pixi install --locked
pixi run setup
pixi run gates
```

Every verb goes through the same route: `pixi run <verb>` → root package script → `bun x turbo run <verb>` → package script.

```console
pixi run setup       # install the Bun workspace and Git hooks
pixi run gates       # lint, type-check, test, actionlint, workflow check, repo checks
pixi run build       # Turbo builds the Rust workspace and documentation stub
pixi run fmt         # apply all formatters
pixi run backlog -- board
pixi run package     # build the conda artifact into output/
```

## What it provides

| Capability | Status | Where |
| :-- | :-- | :-- |
| Authored release policy | Specified in `doc-3` §4; the file is not in this repository yet | `doc-3` §4, `crates/xtask/fixtures/consumers/` |
| Immutable release plans | `plan` derives canonical JSON from the policy and two Git facts | `pixi-release-core`, `doc-1` |
| Git safety for prepare and cut | `prepare --dry-run` records edits without writing; `cut` validates and never publishes | `pixi-release-git`, `decision-4` |
| Consumer release workflows | Golden template in place; rendering planned. Generated files are never hand-edited | `crates/xtask/templates/workflows/`, `doc-2`, `decision-3` |
| Release train | `autorelease` derives the version from Convco and tags; `release` builds the conda
artifact and publishes it to the project channel through OIDC | `.github/workflows/`, `decision-14` |
| Artifact verification and resumable publish | Planned | `doc-2` §3, `doc-3` §2.6 |
| Repository gates | Defined and run in CI: lint, type-check, test, actionlint, workflow and repo checks | `pixi.toml`, `crates/xtask` |

Release evidence is anchored on the annotated tag. Generated workflows are marked, stamped, and drift-gated, and they never float a production pin.

## Stability and platforms

- **Stable:** nothing yet. Every lifecycle command is a scaffold.
- **Toolchain:** the Pixi lock targets `linux-64` for development. Native release artifacts are built on their own runners, not on the development machine.
- **Consumer matrices:** the example consumer workflows in `doc-2` include macOS and Windows runners. The controller itself is built only on its own development lockfile.

The authoritative list of what each command may claim is `doc-1` and `doc-2` §4.

## Repository layout

This repository is a single-root Cargo workspace plus a Bun workspace, both driven from Pixi.

| Path | Role |
| :-- | :-- |
| `crates/pixi-release-core` | Release-domain types and deterministic planning invariants. |
| `crates/pixi-release-git` | Semantic Git port (`GitProtocol`) plus a stateful fake for unit tests. |
| `crates/pixi-release` | The `pixi-release` executable: `clap` grammar, policy load, and presentation. |
| `crates/xtask` | Owner-lane repository automation: Rust checks, the TypeScript workspace scripts, golden templates, and consumer fixtures. It is not the product, and it is deleted once a pinned prior release can do its job. |
| `crates/package.json` | Turbo façade for Cargo, covering every member. |
| `crates/xtask/templates/workflows/` | Golden renderer templates for the consumer lane. |
| `crates/xtask/fixtures/consumers/` | Miniature consumer repositories in the shapes the renderer must satisfy. |
| `crates/xtask/scripts/` | TypeScript workspace-contract checks and their tests, run by `pixi run build`, `typecheck`, and `test`. |
| `backlog/` | Canonical tasks, milestones, decisions, and project documents, managed only through Backlog.md. |
| `.agents/skills/` | Agent skills for this repository, including `session` for work sessions. |

## Development

```bash
pixi run setup       # Bun install and Git hooks
pixi run lint        # Turbo lint
pixi run typecheck   # Turbo type-check
pixi run test        # Turbo test (Rust and TypeScript)
pixi run build       # Turbo build
pixi run fmt         # Turbo format
pixi run gates       # every required pre-merge check
```

The committed `pixi.lock`, `bun.lock`, and `Cargo.lock` are the three resolution records. Do not regenerate a lockfile incidentally; refresh it intentionally and review the diff. Scope Turbo work with a filter: `pixi run test -- --filter=@pixi-release/rust`.

## Contributing

Work is tracked in Backlog.md. Read [`AGENTS.md`](AGENTS.md) before changing project structure or delivery policy.

Before you open a pull request:

- Run `pixi run gates`.
- Record a design change as a decision in `backlog/decisions/` before you implement it.
- Write commit messages in Conventional Commits form. Convco checks them.
- Merge with a normal merge commit: `gh pr merge <number> --merge`. Squash and rebase merges are not used here.

## License

Licensed under either the [MIT License](LICENSE-MIT) or the [Apache License 2.0](LICENSE-APACHE), at your option.
