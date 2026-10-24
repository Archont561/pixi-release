//! Dry-run preparation: it names the files it would change and changes none of them (REL-4).

use pixi_release_core::plan::Version;
use pixi_release_core::prepare::{changelog_target, stage, version_edits};
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

/// AC#3: `prepare` may stage only the changelog path the policy names; asking for no path means
/// the policy's own, and asking for that path is the same request spelled out.
#[rstest]
#[case(None, "CHANGELOG.md")]
#[case(Some("CHANGELOG.md"), "CHANGELOG.md")]
fn the_policy_changelog_path_is_the_only_target(
    #[case] requested: Option<&str>,
    #[case] expected: &str,
) {
    let target = changelog_target("CHANGELOG.md", requested).expect("the policy's own path");

    assert_eq!(target, expected);
}

/// AC#3: a path the policy does not list is refused by name, not quietly redirected.
#[rstest]
fn a_changelog_path_the_policy_does_not_name_is_refused() {
    let refusal = changelog_target("CHANGELOG.md", Some("docs/CHANGELOG.md"))
        .expect_err("the policy names CHANGELOG.md");

    assert_eq!(refusal.key(), "version.changelog");
    assert!(
        refusal.detail().contains("docs/CHANGELOG.md"),
        "{}",
        refusal.detail()
    );
    assert!(
        refusal.detail().contains("CHANGELOG.md"),
        "the refusal must say which path the policy does name: {}",
        refusal.detail()
    );
}

proptest! {
    /// The rule is a comparison, so it must hold for every pair: the request is answered with the
    /// policy's path exactly when the two agree (surrounding whitespace aside), and refused
    /// otherwise — never rewritten into a path the caller did not ask for.
    #[test]
    fn only_the_named_path_is_answered(
        policy in "[A-Za-z0-9_./-]{1,40}",
        requested in "[A-Za-z0-9_./-]{1,40}",
        pad in 0usize..3,
    ) {
        let policy = format!("{}{}", " ".repeat(pad), policy);
        let outcome = changelog_target(&policy, Some(&requested));

        match outcome {
            Ok(target) => prop_assert_eq!(target, policy.trim().to_owned()),
            Err(refusal) => {
                prop_assert_eq!(refusal.key(), "version.changelog");
                prop_assert_ne!(requested.trim(), policy.trim());
            }
        }
    }
}

/// AC#1: writing an edit leaves the manifest reading as the planned version to the very scanner
/// that found the old literal — and touches one line, not the document.
#[rstest]
#[case(CARGO)]
#[case(PACKAGE_JSON)]
fn staging_a_manifest_reaches_the_planned_version(#[case] text: &str) {
    let planned = version("0.2.0");
    let edit = version_edits([("manifest".to_owned(), text.to_owned())], &planned)
        .expect("the fixture carries a literal")
        .remove(0);

    let staged = stage(text, &edit);

    assert!(
        version_edits([("manifest".to_owned(), staged.clone())], &planned)
            .expect("the staged manifest still carries a literal")
            .is_empty(),
        "the staged manifest still needs an edit:\n{staged}"
    );
    assert_eq!(staged.lines().count(), text.lines().count());
    assert_eq!(
        staged
            .lines()
            .zip(text.lines())
            .filter(|(staged, original)| staged != original)
            .count(),
        1,
        "exactly one line may move:\n{staged}"
    );
}

proptest! {
    /// Staging is a replacement of one literal, so it must not depend on how many times it runs:
    /// staging an already staged manifest is a no-op, and the version it reaches is the plan's.
    #[test]
    fn staging_is_idempotent(
        major in 0u64..40,
        minor in 0u64..40,
        patch in 0u64..40,
    ) {
        let planned = Version::parse(&format!("{major}.{minor}.{patch}")).expect("semver");
        let edit = version_edits([("manifest".to_owned(), CARGO.to_owned())], &planned)
            .expect("the fixture carries a literal")
            .remove(0);

        let once = stage(CARGO, &edit);
        let twice = stage(&once, &edit);

        prop_assert_eq!(&once, &twice);
        prop_assert!(version_edits([("manifest".to_owned(), once.clone())], &planned)
            .expect("still a literal")
            .is_empty());
    }
}
