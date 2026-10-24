# Offline sandbox (orphan branch)

Built 2026-10-24T19:44:05Z from commit `aa75666` for platform `linux-64`.
`pixi.lock` sha256 `0a81791080a73cc3c429c61f950c5dddc6e4782ff06434be0fcef7f48ee7c68d`.

The verified self-bootstrap binary is stored at `.pixi-sandbox/tools/linux-64/pixi-sandbox`. The branch root intentionally contains documentation only.

| env | platform | packed | unpacked | files |
| --- | --- | ---: | ---: | ---: |
| `default` | linux-64 | 400.3 MiB | 1694.7 MiB | 48 |

Cargo dependencies: **215 crates**, 273.8 MiB (loose) from `Cargo.lock` sha256 `321ef750dbaa…`; restore materialises them to `.pixi-sandbox/vendor/`. Built with cargo 1.99.0 (5f94df478 2026-08-27); rustc 1.99.0 (b940084d7 2026-09-28).


## Restore on the disconnected machine

```bash
./.pixi-sandbox/tools/linux-64/pixi-sandbox doctor --branch-location . --verify
./.pixi-sandbox/tools/linux-64/pixi-sandbox restore --branch-location . --output-path <project> --force
# then, from <project> with no network, use pixi as the sole entrypoint:
pixi install --frozen --offline
pixi run --frozen -- cargo build --offline
```

Every manifest blob is verified before it is written into the working tree.
