# Consumer fixtures

Materialised miniature consumer repositories. The renderer is wrong if it works for only one
of these, and all three shapes exist today in `Archont561/*` — see `doc-3` §3 for the matrix
and `doc-4` for the survey they are drawn from.

| Fixture | Shape | What the renderer must not assume |
|---|---|---|
| `single-root-cargo/` | single-root Cargo, no Turbo, one JS package (`pixi-sandbox`, `nadir`) | that `bun x turbo` exists |
| `polyglot-turbo/` | `apps/` + `packages/` + `crates/` + `python/`, Turbo (`geoquery`, `pathway`, `qgis-rs`) | that one `cargo build` is the whole build |
| `per-package-pixi/` | per-package `pixi.toml`, no Cargo workspace (`geoaiworkbench`) | that a root `Cargo.toml` exists |

These are **fixtures, not products**: each is the smallest tree that makes `init`, `workflow
render` and `workflow check` answer differently. They are what gives the golden-workflow tests
meaning — `pixi-sandbox` carries the same thing at
`crates/pixi-sandbox/tests/fixtures/transport/`.

Each fixture holds, at minimum, its `.pixi-release.toml` and whichever manifests the renderer
must read. Expected renderer output lands beside it as `expected/` once `workflow render`
exists (REL-4); until then the inputs are checked in so the shapes cannot drift from the
survey while the renderer is being written.
