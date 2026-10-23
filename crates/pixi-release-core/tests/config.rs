//! Release policy parsing and validation (`doc-3` §4, REL-2).
//!
//! Every case starts from the documented policy shape and mutates one thing, so a refusal proves
//! the rule rather than a fixture typo.

use std::collections::BTreeMap;

use pixi_release_core::config::{ConvcoEvidence, PolicyContext, ReleasePolicy, VersionSource};
use proptest::prelude::*;
use rstest::{fixture, rstest};
use rstest_bdd_macros::{scenario, then};

/// `doc-3` §4's shape, with this repository's own values.
const DOCUMENTED: &str = r#"
schema = 1

[project]
version-source = "convco"
tag-format     = "v{version}"
bot-identity   = "pixi-release[bot]"

[version]
manifests = ["Cargo.toml", "pixi.toml", "package.json"]
lockfiles = ["Cargo.lock", "bun.lock", "pixi.lock"]
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

#[fixture]
fn documented() -> &'static str {
    DOCUMENTED
}

#[fixture]
fn convco_configured() -> PolicyContext {
    PolicyContext {
        convco_configured: true,
    }
}

/// The documented policy with one replacement applied.
fn mutate(from: &str, to: &str) -> String {
    DOCUMENTED.replace(from, to)
}

/// Asserts the documented policy is accepted.
#[rstest]
fn documented_policy_is_accepted(documented: &str, convco_configured: PolicyContext) {
    let policy = ReleasePolicy::parse_with(documented, &convco_configured)
        .unwrap_or_else(|errors| panic!("documented policy should parse: {errors}"));

    assert_eq!(policy.version_source(), VersionSource::Convco);
    assert_eq!(policy.tag_format(), "v{version}");
    assert_eq!(policy.manifests().len(), 3);
    assert_eq!(policy.targets().len(), 1);
}

/// AC#1: an unsupported schema is refused by name.
#[rstest]
fn unsupported_schema_names_the_schema_key(convco_configured: PolicyContext) {
    let text = mutate("schema = 1", "schema = 99");
    let errors = ReleasePolicy::parse_with(&text, &convco_configured)
        .expect_err("schema 99 must be refused");

    assert_eq!(errors.first_key(), "schema");
}

/// AC#1: an unknown key is named rather than silently ignored.
#[rstest]
fn unknown_key_is_named(convco_configured: PolicyContext) {
    let text = mutate(
        "tag-format     = \"v{version}\"",
        "tag-format     = \"v{version}\"\nunknown-key = true",
    );
    let errors = ReleasePolicy::parse_with(&text, &convco_configured)
        .expect_err("an unknown key must be refused");

    assert!(
        errors.to_string().contains("unknown-key"),
        "error should name the offending key: {errors}"
    );
}

/// AC#1: a version source outside the accepted set names `project.version-source`.
#[rstest]
fn unknown_version_source_names_the_key(convco_configured: PolicyContext) {
    let text = mutate(
        "version-source = \"convco\"",
        "version-source = \"git-tag\"",
    );
    let errors = ReleasePolicy::parse_with(&text, &convco_configured)
        .expect_err("git-tag is not an accepted version source");

    assert_eq!(errors.first_key(), "project.version-source");
}

/// AC#3: the native `SemVer` source is refused while a Convco policy is active.
#[rstest]
fn native_semver_is_refused_while_convco_is_active(convco_configured: PolicyContext) {
    let text = mutate(
        "version-source = \"convco\"",
        "version-source = \"manifest\"",
    );
    let errors = ReleasePolicy::parse_with(&text, &convco_configured)
        .expect_err("manifest source must be refused beside an active Convco policy");

    assert_eq!(errors.first_key(), "project.version-source");
    assert!(
        errors.to_string().contains("decision-6"),
        "the refusal should cite the authority decision: {errors}"
    );
}

/// The same policy is accepted once no Convco configuration exists, which proves the rule is
/// about the combination rather than about the value.
#[rstest]
fn native_semver_is_accepted_without_convco_configuration() {
    let text = mutate(
        "version-source = \"convco\"",
        "version-source = \"manifest\"",
    );
    let context = PolicyContext {
        convco_configured: false,
    };
    let policy = ReleasePolicy::parse_with(&text, &context)
        .unwrap_or_else(|errors| panic!("manifest source alone is valid: {errors}"));

    assert_eq!(policy.version_source(), VersionSource::Manifest);
}

/// AC#1: a tag format that cannot carry the version is refused by key.
#[rstest]
fn tag_format_without_placeholder_names_the_key(convco_configured: PolicyContext) {
    let text = mutate("v{version}", "release");
    let errors = ReleasePolicy::parse_with(&text, &convco_configured)
        .expect_err("a tag format with no {version} placeholder is unsafe");

    assert_eq!(errors.first_key(), "project.tag-format");
}

/// `decision-9`: only `github-release` is admissible before REL-6.
#[rstest]
fn registry_target_names_the_key_and_cites_the_decision(convco_configured: PolicyContext) {
    let text = format!("{DOCUMENTED}\n[[target]]\nname = \"prefix\"\nrequired = false\n");
    let errors = ReleasePolicy::parse_with(&text, &convco_configured)
        .expect_err("a registry target before REL-6 must be refused");

    assert_eq!(errors.first_key(), "target.name");
    assert!(errors.to_string().contains("decision-9"));
}

/// `decision-10` §2: the renderer claims the release lane only.
#[rstest]
fn ci_in_the_render_list_names_the_key(convco_configured: PolicyContext) {
    let text = mutate(
        "render = [\"auto-release\", \"release\"]",
        "render = [\"ci\", \"auto-release\", \"release\"]",
    );
    let errors = ReleasePolicy::parse_with(&text, &convco_configured)
        .expect_err("ci.yml stays hand-written (decision-10)");

    assert_eq!(errors.first_key(), "workflows.render");
}

/// AC#1: an empty manifest list is refused, because `prepare` would own nothing.
#[rstest]
fn empty_manifest_list_names_the_key(convco_configured: PolicyContext) {
    let text = mutate(
        "manifests = [\"Cargo.toml\", \"pixi.toml\", \"package.json\"]",
        "manifests = []",
    );
    let errors = ReleasePolicy::parse_with(&text, &convco_configured)
        .expect_err("an empty manifest list leaves prepare nothing to own");

    assert_eq!(errors.first_key(), "version.manifests");
}

/// AC#1: a repeated manifest is ambiguous about which file owns the version literal.
#[rstest]
fn duplicate_manifest_names_the_key(convco_configured: PolicyContext) {
    let text = mutate(
        "manifests = [\"Cargo.toml\", \"pixi.toml\", \"package.json\"]",
        "manifests = [\"Cargo.toml\", \"Cargo.toml\"]",
    );
    let errors = ReleasePolicy::parse_with(&text, &convco_configured)
        .expect_err("a repeated manifest is ambiguous");

    assert_eq!(errors.first_key(), "version.manifests");
}

/// AC#2: Convco mode records the tool version and configuration hashes as evidence.
#[rstest]
fn convco_mode_records_version_and_configuration_hashes(
    documented: &str,
    convco_configured: PolicyContext,
) {
    let policy = ReleasePolicy::parse_with(documented, &convco_configured)
        .expect("documented policy should parse");
    let mut hashes = BTreeMap::new();
    hashes.insert(".versionrc".to_owned(), "a".repeat(64));
    let evidence = ConvcoEvidence::new("0.7.2", hashes).expect("evidence should be valid");

    let plan = policy
        .plan_evidence(Some(evidence))
        .expect("convco mode with evidence should produce plan evidence");

    let convco = plan.convco().expect("convco evidence is recorded");
    assert_eq!(convco.version(), "0.7.2");
    assert_eq!(convco.config_hashes().len(), 1);
    assert_eq!(plan.version_source(), VersionSource::Convco);
}

/// AC#2: Convco mode without evidence is refused, because the plan would be unreproducible.
#[rstest]
fn convco_mode_without_evidence_is_refused(documented: &str, convco_configured: PolicyContext) {
    let policy = ReleasePolicy::parse_with(documented, &convco_configured)
        .expect("documented policy should parse");
    let error = policy
        .plan_evidence(None)
        .expect_err("convco mode requires evidence");

    assert_eq!(error.key(), "project.version-source");
}

/// AC#3, evidence half: manifest mode refuses Convco evidence rather than ignoring it.
#[rstest]
fn manifest_mode_refuses_convco_evidence() {
    let text = mutate(
        "version-source = \"convco\"",
        "version-source = \"manifest\"",
    );
    let context = PolicyContext {
        convco_configured: false,
    };
    let policy = ReleasePolicy::parse_with(&text, &context).expect("manifest mode should parse");
    let mut hashes = BTreeMap::new();
    hashes.insert(".versionrc".to_owned(), "b".repeat(64));
    let evidence = ConvcoEvidence::new("0.7.2", hashes).expect("evidence should be valid");

    let error = policy
        .plan_evidence(Some(evidence))
        .expect_err("manifest mode must not carry convco evidence");

    assert_eq!(error.key(), "project.version-source");
}

proptest! {
    /// A tag format without the placeholder is never accepted (decision-4 keeps the tag the
    /// evidence anchor, so it must carry the version).
    #[test]
    fn any_tag_format_without_the_placeholder_is_refused(format in "[a-zA-Z0-9._/-]{1,12}") {
        prop_assume!(!format.contains("{version}"));
        let text = DOCUMENTED.replace("v{version}", &format);
        let errors = ReleasePolicy::parse_with(
            &text,
            &PolicyContext { convco_configured: true },
        )
        .expect_err("a placeholder-free tag format must be refused");
        prop_assert_eq!(errors.first_key(), "project.tag-format");
    }

    /// Parsing is deterministic: the same text yields the same typed policy.
    #[test]
    fn parsing_is_deterministic(schema in 1u32..=1) {
        let text = DOCUMENTED.replace("schema = 1", &format!("schema = {schema}"));
        let context = PolicyContext { convco_configured: true };
        let first = ReleasePolicy::parse_with(&text, &context).expect("schema 1 should parse");
        let second = ReleasePolicy::parse_with(&text, &context).expect("schema 1 should parse");
        prop_assert_eq!(first, second);
    }
}

/// The policy for one named case, or a panic naming the case.
fn case_policy(case: &str) -> String {
    match case {
        "documented" => DOCUMENTED.to_owned(),
        "bad-target" => format!("{DOCUMENTED}\n[[target]]\nname = \"prefix\"\n"),
        "bad-format" => DOCUMENTED.replace("v{version}", "release"),
        "bad-source" => DOCUMENTED.replace(
            "version-source = \"convco\"",
            "version-source = \"git-tag\"",
        ),
        other => panic!("unknown policy case: {other}"),
    }
}

/// Shared assertion so the step and the scenario body check the same thing.
fn assert_case(case: &str, outcome: &str) {
    let context = PolicyContext {
        convco_configured: true,
    };
    let result = ReleasePolicy::parse_with(&case_policy(case), &context);
    if outcome == "accepted" {
        assert!(result.is_ok(), "case {case} should be accepted: {result:?}");
        return;
    }
    let errors = result.expect_err("case should be refused");
    assert!(
        errors.iter().any(|error| error.key() == outcome),
        "case {case} should name {outcome}, got {errors}"
    );
}

#[then("policy case {case} yields {outcome}")]
fn policy_case(case: String, outcome: String) {
    assert_case(&case, &outcome);
}

#[scenario(path = "tests/features/policy_validation.feature")]
fn policy_validation(case: String, outcome: String) {
    assert_case(&case, &outcome);
}
