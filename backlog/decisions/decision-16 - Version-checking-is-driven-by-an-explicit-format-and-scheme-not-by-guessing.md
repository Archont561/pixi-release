---
id: decision-16
title: 'Version checking is driven by an explicit format and scheme, not by guessing'
date: '2026-10-24 18:10'
status: proposed
---
## Context

`prepare` reads a version literal with a line scan (`crates/pixi-release-core/src/prepare.rs::version_value`)
that understands one TOML shape and one JSON shape. Everything else it touches, it guesses at.

Three consequences are already measurable in this repository, before any new ecosystem is added:

- A YAML line (`version: "0.1.0"`, as in `crates/pixi-release/recipe/recipe.yaml`) is not `version =`
  or `"version":`, so the scan skips it. Listing the recipe in `version.manifests` makes `prepare`
  refuse the manifest as *carrying no version literal*, and no test lists it. The recipe's version is
  enforced instead by an owner-lane check (`crates/xtask/src/main.rs::packaging_agrees`), outside the
  product.
- `Version` derives `Ord` on its `String` payload, so `"0.10.0" < "0.9.0"`. Nothing compares versions
  today, so the defect is latent; the type still promises an ordering every future caller will trust.
- The syntax check accepts strict semver only, and refuses versions the registries this product is
  meant to serve consider normal: `1.2.3.4` (NuGet), `1.2.3-SNAPSHOT` (Maven), `0.1.0rc1` (PEP 440).

The wider context is that the repositories this product is modelled on each wrote their own answer:
`qgis-sdk` ships a hand-written `.versionrc` parser and per-format stampers in `scripts/release.py`;
`pixi-sandbox` ships `workspace_version` (a real TOML parse) plus per-manifest stamping in
`crates/xtask/src/prepare_release.rs`. Neither approach is wrong, and both are local.

## Decision

A manifest entry declares how to read and write its version. The schema records three facts per
manifest, and the reader refuses an entry that does not supply them:

- `format` — how to parse the document: `toml`, `json`, `yaml`, `xml`, or `go-tag`.
- `key` — which declaration is the version, addressed structurally (`project/version`, not "the first
  line that looks like a version"). `go-tag` is the explicit statement that no literal exists: the tag
  is the source, and the check is the module path's major suffix against the tag's major.
- `scheme` — how to normalise and compare: `semver`, `pep440`, `nuget`, `maven`, `conda`.

The tag stays the canonical serialisation of a release (`decision-4`); a scheme renders that version
into its manifest's dialect and parses it back for comparison. Comparison is by parsed components per
scheme, never by the raw string, and `Version` stops deriving an ordering it does not honour.

## Consequences

- Schema 2 replaces `version.manifests` (a list of paths) with `[[version.manifest]]` entries. REL-11
  owns the schema change; REL-28 owns the checker that consumes it.
- The first four formats are `toml`, `json`, `yaml` and `go-tag`, in that order of urgency: the first
  two are in use, YAML is the recipe this repository already ships, and `go-tag` is the first format
  where the correct answer is "there is no literal".
- `xml` (Maven, NuGet) is admitted when a consumer needs it, with its own fixture — a structural XML
  reader, never a line scan.
- Owner-lane checks that duplicate the checker (`xtask::versions_agree`, `xtask::packaging_agrees`)
  are deleted once the checker is callable from the product; keeping both is the second copy this
  decision exists to prevent.
- `decision-7` is untouched: one version per repository still holds for a single-artifact repository.
  Multi-artifact repositories are `decision-17`.
