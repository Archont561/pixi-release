//! The immutable release plan (`doc-2` §2.3, REL-3).
//!
//! The plan is evidence, so every case here is about a property a reviewer can check: the same
//! inputs give the same bytes, the bytes name where the version came from, and nothing in the
//! proposal depends on the order the policy happened to list its targets in.

use std::collections::BTreeMap;

use pixi_release_core::config::{ConvcoEvidence, PolicyContext, ReleasePolicy};
use pixi_release_core::plan::{
    Bump, Plan, PlanInputs, PolicyDigest, SourceRevision, TargetPlan, Version, VersionAuthority,
    normalize_targets,
};
use proptest::prelude::*;
use rstest::{fixture, rstest};

/// A policy that admits one target, so the plan has exactly one row to normalise.
const POLICY: &str = r#"
schema = 1

[project]
version-source = "convco"
tag-format     = "v{version}"
bot-identity   = "pixi-release[bot]"

[version]
manifests = ["Cargo.toml"]
lockfiles = ["Cargo.lock"]
changelog = "CHANGELOG.md"

[gates]
required = ["gates"]

[workflows]
render = ["auto-release", "release"]
runner = "ubuntu-latest"

[[target]]
name     = "github-release"
required = true
"#;

/// The same policy with a manifest version source, for the authority-compatibility case.
fn manifest_policy() -> String {
    POLICY.replace(
        "version-source = \"convco\"",
        "version-source = \"manifest\"",
    )
}

#[fixture]
fn policy() -> ReleasePolicy {
    ReleasePolicy::parse_with(
        POLICY,
        &PolicyContext {
            convco_configured: true,
        },
    )
    .expect("the fixture policy is valid")
}

/// Convco evidence with one configuration file, so the recorded digest is predictable.
fn convco_evidence() -> ConvcoEvidence {
    let mut config = BTreeMap::new();
    config.insert(".versionrc".to_owned(), "ab".repeat(32));
    ConvcoEvidence::new("0.7.2", config).expect("evidence with a version and one config file")
}

#[fixture]
fn source() -> SourceRevision {
    SourceRevision::new("9f2c1d4b7a8e6f5d3c2b1a09f8e7d6c5b4a39281", true)
        .expect("the fixture revision is a sha")
}

#[fixture]
fn digest() -> PolicyDigest {
    PolicyDigest::new("sha256:6f1c1d0b8f5f3f8f4a1e5e7c2b9d0a3f4c5e6b7a8d9e0f1a2b3c4d5e6f708192")
        .expect("the fixture digest is sha256 hex")
}

#[fixture]
fn version() -> Version {
    Version::parse("0.2.0").expect("the fixture version is semver")
}

/// Builds the documented plan: explicit version, one admissible target, fixed evidence.
fn explicit_plan(policy: &ReleasePolicy, source: SourceRevision, digest: &PolicyDigest) -> Plan {
    Plan::propose(
        policy,
        PlanInputs {
            source,
            policy_digest: digest.as_str().to_owned(),
            tool_version: "0.1.0".into(),
        },
        VersionAuthority::Explicit {
            version: Version::parse("0.2.0").expect("fixture version"),
        },
    )
    .expect("the documented plan is proposed")
}

/// `DoD` #2: a golden fixture, materialised from bounded components, is the shape reviewers read.
#[rstest]
fn the_golden_plan_is_byte_identical_to_the_checked_in_fixture(
    policy: ReleasePolicy,
    source: SourceRevision,
    digest: PolicyDigest,
) {
    let plan = explicit_plan(&policy, source, &digest);

    assert_eq!(
        plan.to_json(),
        include_str!("fixtures/plan.golden.json"),
        "regenerate only when the plan shape is deliberately changed"
    );
}

/// AC#1: identical inputs produce byte-identical JSON, including on repeat.
#[rstest]
fn identical_inputs_produce_identical_json(
    policy: ReleasePolicy,
    source: SourceRevision,
    digest: PolicyDigest,
) {
    let first = explicit_plan(&policy, source.clone(), &digest);
    let second = explicit_plan(&policy, source, &digest);

    assert_eq!(first.to_json(), second.to_json());
    assert_eq!(first.to_json(), first.to_json());
}

/// AC#2: an explicit plan records who chose the version, and the revision it was derived from.
#[rstest]
fn an_explicit_plan_names_the_authority_and_the_source(
    policy: ReleasePolicy,
    source: SourceRevision,
    digest: PolicyDigest,
) {
    let plan = explicit_plan(&policy, source, &digest);
    let json = plan.to_json();

    assert_eq!(plan.version().as_str(), "0.2.0");
    assert_eq!(plan.tag(), "v0.2.0");
    assert!(json.contains("\"kind\": \"explicit\""), "{json}");
    assert!(json.contains("\"requested\": \"0.2.0\""), "{json}");
    assert!(
        json.contains("9f2c1d4b7a8e6f5d3c2b1a09f8e7d6c5b4a39281"),
        "{json}"
    );
    assert!(json.contains("\"clean\": true"), "{json}");
    assert!(
        json.contains(&format!("\"digest\": \"{}\"", digest.as_str())),
        "{json}"
    );
}

/// AC#2, Convco half: a derived plan records the tool, the bump and the config it read.
#[rstest]
fn a_convco_plan_records_the_tool_bump_and_config_hashes(
    policy: ReleasePolicy,
    source: SourceRevision,
    digest: PolicyDigest,
) {
    let evidence = convco_evidence();
    let plan = Plan::propose(
        &policy,
        PlanInputs {
            source,
            policy_digest: digest.as_str().to_owned(),
            tool_version: "0.1.0".into(),
        },
        VersionAuthority::Convco {
            bump: Bump::Minor,
            version: Version::parse("0.3.0").expect("derived version"),
            evidence,
        },
    )
    .expect("a convco plan is proposed");

    let json = plan.to_json();
    assert!(json.contains("\"kind\": \"convco\""), "{json}");
    assert!(json.contains("\"bump\": \"minor\""), "{json}");
    assert!(json.contains("\"version\": \"0.7.2\""), "{json}");
    assert!(json.contains("\"config\""), "{json}");
    assert!(json.contains(&"ab".repeat(32)), "{json}");
}

/// The authority must match the policy: a Convco-derived version under a manifest policy is refused.
#[rstest]
fn a_manifest_policy_refuses_a_convco_derived_version(
    source: SourceRevision,
    digest: PolicyDigest,
) {
    let policy = ReleasePolicy::parse(&manifest_policy()).expect("manifest policy parses");
    let evidence = convco_evidence();

    let error = Plan::propose(
        &policy,
        PlanInputs {
            source,
            policy_digest: digest.as_str().to_owned(),
            tool_version: "0.1.0".into(),
        },
        VersionAuthority::Convco {
            bump: Bump::Patch,
            version: Version::parse("0.2.1").expect("version"),
            evidence,
        },
    )
    .expect_err("manifest mode cannot carry a Convco derivation");

    assert_eq!(error.key(), "project.version-source");
}

/// A malformed version is refused before it reaches a tag.
#[rstest]
#[case("0.2")]
#[case("v0.2.0")]
#[case("0.02.0")]
#[case("")]
fn a_version_that_is_not_semver_is_refused(#[case] candidate: &str) {
    let error = Version::parse(candidate).expect_err("not semver");
    assert_eq!(error.key(), "version");
}

/// A revision is a commit id, not free text.
#[rstest]
#[case("")]
#[case("main")]
#[case("abc123")]
#[case("9f2c1d4b7a8e6f5d3c2b1a09f8e7d6c5b4a3928155")]
fn a_revision_that_is_not_a_commit_id_is_refused(#[case] candidate: &str) {
    let error = SourceRevision::new(candidate, true).expect_err("not a commit id");
    assert_eq!(error.key(), "source.commit");
}

/// A policy digest carries its algorithm, so a bare hex string cannot masquerade as one.
#[rstest]
#[case("")]
#[case("6f1c1d0b")]
#[case("sha256:xyz")]
#[case("sha1:6f1c1d0b8f5f3f8f4a1e5e7c2b9d0a3f4c5e6b7a8d9e0f1a2b3c4d5e6f708192")]
fn a_digest_that_is_not_sha256_hex_is_refused(#[case] candidate: &str) {
    let error = PolicyDigest::new(candidate).expect_err("not a sha256 digest");
    assert_eq!(error.key(), "policy.digest");
}

/// A plan with no target publishes nowhere, so it is not a plan.
#[rstest]
fn a_plan_without_targets_is_refused(source: SourceRevision, digest: PolicyDigest) {
    let policy = ReleasePolicy::parse(&POLICY.replace(
        "[[target]]\nname     = \"github-release\"\nrequired = true\n",
        "",
    ))
    .expect("policy without targets still parses");

    let error = Plan::propose(
        &policy,
        PlanInputs {
            source,
            policy_digest: digest.as_str().to_owned(),
            tool_version: "0.1.0".into(),
        },
        VersionAuthority::Explicit {
            version: Version::parse("0.2.0").expect("version"),
        },
    )
    .expect_err("a plan needs a target");

    assert_eq!(error.key(), "target.name");
}

/// `DoD` #1: target rows are canonical — sorted by name, one row per name.
#[rstest]
fn target_rows_are_sorted_and_deduplicated() {
    let rows = normalize_targets([
        ("prefix".to_owned(), true),
        ("github-release".to_owned(), true),
        ("prefix".to_owned(), true),
    ])
    .expect("consistent duplicates are collapsed");

    assert_eq!(
        rows,
        vec![
            TargetPlan::new("github-release", true),
            TargetPlan::new("prefix", true),
        ]
    );
}

/// Duplicates that disagree are ambiguous, not silently merged.
#[rstest]
fn conflicting_duplicates_are_refused() {
    let error = normalize_targets([("prefix".to_owned(), true), ("prefix".to_owned(), false)])
        .expect_err("a name cannot be both required and optional");

    assert_eq!(error.key(), "target.name");
}

proptest! {
    /// AC#1 and DoD #1: the order a policy listed its targets in never changes the plan.
    #[test]
    fn target_order_does_not_change_the_plan(
        names in prop::collection::vec("[a-z][a-z0-9-]{0,8}", 1..6),
        required in any::<bool>(),
    ) {
        let rows: Vec<(String, bool)> = names.iter().map(|name| (name.clone(), required)).collect();
        let forwards = normalize_targets(rows.clone()).expect("consistent rows");
        let mut reversed = rows;
        reversed.reverse();
        let backwards = normalize_targets(reversed).expect("consistent rows");

        prop_assert_eq!(forwards, backwards);
    }

    /// `DoD` #1: every normalised set has distinct names, in ascending order.
    #[test]
    fn normalised_rows_have_distinct_sorted_names(
        names in prop::collection::vec("[a-z][a-z0-9-]{0,8}", 0..8),
        required in any::<bool>(),
    ) {
        let rows = normalize_targets(
            names
                .iter()
                .map(|name| (name.clone(), required))
                .collect::<Vec<_>>(),
        )
        .expect("consistent rows");

        prop_assert!(rows.windows(2).all(|pair| pair[0].name() < pair[1].name()));
    }

    /// AC#1: serialisation is a pure function of the inputs, whatever the version.
    #[test]
    fn serialisation_is_a_pure_function_of_the_inputs(
        major in 0u64..100,
        minor in 0u64..100,
        patch in 0u64..100,
    ) {
        let policy = ReleasePolicy::parse(POLICY).expect("fixture policy");
        let source = SourceRevision::new("9f2c1d4b7a8e6f5d3c2b1a09f8e7d6c5b4a39281", false)
            .expect("fixture revision");
        let digest = PolicyDigest::new(
            "sha256:6f1c1d0b8f5f3f8f4a1e5e7c2b9d0a3f4c5e6b7a8d9e0f1a2b3c4d5e6f708192",
        )
        .expect("fixture digest");
        let version = Version::parse(&format!("{major}.{minor}.{patch}")).expect("semver");

        let propose = || {
            Plan::propose(
                &policy,
                PlanInputs {
                    source: source.clone(),
                    policy_digest: digest.as_str().to_owned(),
                    tool_version: "0.1.0".into(),
                },
                VersionAuthority::Explicit {
                    version: version.clone(),
                },
            )
            .expect("plan is proposed")
            .to_json()
        };

        prop_assert_eq!(propose(), propose());
    }

    /// A tag is exactly the policy format with the version substituted, never anything else.
    #[test]
    fn the_tag_is_the_policy_format_with_the_version_substituted(
        major in 0u64..50,
        minor in 0u64..50,
        patch in 0u64..50,
    ) {
        let policy = ReleasePolicy::parse(POLICY).expect("fixture policy");
        let source = SourceRevision::new("9f2c1d4b7a8e6f5d3c2b1a09f8e7d6c5b4a39281", true)
            .expect("fixture revision");
        let digest = PolicyDigest::new(
            "sha256:6f1c1d0b8f5f3f8f4a1e5e7c2b9d0a3f4c5e6b7a8d9e0f1a2b3c4d5e6f708192",
        )
        .expect("fixture digest");
        let version = Version::parse(&format!("{major}.{minor}.{patch}")).expect("semver");

        let plan = Plan::propose(
            &policy,
            PlanInputs {
                source,
                policy_digest: digest.as_str().to_owned(),
                tool_version: "0.1.0".into(),
            },
            VersionAuthority::Explicit {
                version: version.clone(),
            },
        )
        .expect("plan is proposed");

        prop_assert_eq!(plan.tag(), format!("v{version}"));
    }
}
