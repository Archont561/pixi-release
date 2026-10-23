//! Dry-run preparation: it names the files it would change and changes none of them (REL-4).

use pixi_release_core::plan::Version;
use pixi_release_core::prepare::version_edits;
use proptest::prelude::*;
use rstest::rstest;

fn version(text: &str) -> Version {
    Version::parse(text).expect("a semver fixture")
}

/// A Cargo manifest already at the old version.
const CARGO: &str = r#"
[package]
name = "demo"
version = "0.1.0"
edition = "2021"
"#;

/// A JSON manifest, whose version literal is spelled differently.
const PACKAGE_JSON: &str = r#"{
  "name": "demo",
  "version": "0.1.0",
  "private": true
}
"#;

/// AC#1: the intended change is reported per file, with both values.
#[rstest]
fn the_intended_change_is_reported_per_file() {
    let edits = version_edits(
        [
            ("Cargo.toml".to_owned(), CARGO.to_owned()),
            ("package.json".to_owned(), PACKAGE_JSON.to_owned()),
        ],
        &version("0.2.0"),
    )
    .expect("both manifests carry a version literal");

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].path(), "Cargo.toml");
    assert_eq!(edits[0].from(), "0.1.0");
    assert_eq!(edits[0].to(), "0.2.0");
    assert_eq!(edits[1].path(), "package.json");
    assert_eq!(edits[1].from(), "0.1.0");
}

/// A manifest already at the target version needs no edit, and is therefore not recorded.
#[rstest]
fn a_manifest_already_at_the_target_version_is_not_recorded() {
    let edits = version_edits(
        [("Cargo.toml".to_owned(), CARGO.to_owned())],
        &version("0.1.0"),
    )
    .expect("the literal parses");

    assert!(edits.is_empty(), "{edits:?}");
}

/// A manifest without a version literal is refused by name, rather than silently skipped.
#[rstest]
fn a_manifest_without_a_version_literal_is_refused() {
    let error = version_edits(
        [(
            "pixi.toml".to_owned(),
            "[workspace]\nname = \"demo\"\n".to_owned(),
        )],
        &version("0.2.0"),
    )
    .expect_err("no literal to replace");

    assert_eq!(error.key(), "version.manifests");
    assert!(error.detail().contains("pixi.toml"), "{}", error.detail());
}

/// Every manifest must yield an edit, so a policy listing several is answered the same way.
#[rstest]
fn the_edits_keep_the_order_the_policy_listed() {
    let edits = version_edits(
        [
            ("b.toml".to_owned(), "version = \"0.1.0\"\n".to_owned()),
            ("a.toml".to_owned(), "version = \"0.1.0\"\n".to_owned()),
        ],
        &version("1.0.0"),
    )
    .expect("both parse");

    assert_eq!(edits[0].path(), "b.toml");
    assert_eq!(edits[1].path(), "a.toml");
}

proptest! {
    /// Whatever the target version, every recorded edit ends at exactly that version.
    #[test]
    fn every_edit_targets_the_planned_version(
        major in 0u64..90,
        minor in 0u64..90,
        patch in 0u64..90,
    ) {
        let planned = Version::parse(&format!("{major}.{minor}.{patch}")).expect("semver");
        let edits = version_edits(
            [("Cargo.toml".to_owned(), CARGO.to_owned())],
            &planned,
        )
        .expect("the manifest has a literal");

        prop_assert!(edits.iter().all(|edit| edit.to() == planned.as_str()));
        prop_assert!(edits.iter().all(|edit| edit.from() != edit.to()) || edits.is_empty());
    }
}
