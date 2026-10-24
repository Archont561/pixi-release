//! `cut` refuses everything it cannot prove (`doc-2` §7), and never publishes (REL-4).

use pixi_release_core::config::{PolicyContext, ReleasePolicy};
use pixi_release_core::cut::{CutPermit, GateReport, cut, permit};
use pixi_release_core::plan::{Bump, Plan, PlanInputs, SourceRevision, Version, VersionAuthority};
use pixi_release_git::{FakeGit, GitOp, ReleaseGit, TagRef};
use proptest::prelude::*;
use rstest::{fixture, rstest};

const HEAD: &str = "9f2c1d4b7a8e6f5d3c2b1a09f8e7d6c5b4a39281";
const DIGEST: &str = "sha256:6f1c1d0b8f5f3f8f4a1e5e7c2b9d0a3f4c5e6b7a8d9e0f1a2b3c4d5e6f708192";

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

/// The plan every case starts from: `HEAD`, a clean tree, the fixture policy digest.
fn plan() -> Plan {
    Plan::propose(
        &ReleasePolicy::parse(POLICY).expect("policy"),
        PlanInputs {
            source: SourceRevision::new(HEAD, true).expect("revision"),
            policy_digest: DIGEST.to_owned(),
            tool_version: "0.1.0".to_owned(),
        },
        VersionAuthority::Convco {
            bump: Bump::Minor,
            version: Version::parse("0.2.0").expect("version"),
            evidence: pixi_release_core::config::ConvcoEvidence::new(
                "0.7.2",
                [(".versionrc".to_owned(), "ab".repeat(32))]
                    .into_iter()
                    .collect(),
            )
            .expect("evidence"),
        },
    )
    .expect("plan")
    .with_touched(["Cargo.toml".to_owned(), "CHANGELOG.md".to_owned()])
}

/// A fake repository after `prepare`: on the planned commit, dirty by exactly the recorded set.
fn git() -> FakeGit {
    let mut git = FakeGit::new(HEAD, true);
    git.set_dirty_paths(["CHANGELOG.md".to_owned(), "Cargo.toml".to_owned()]);
    git
}

/// The same fixture before `prepare` ran: clean, so there is nothing for `cut` to commit.
fn unprepared() -> FakeGit {
    FakeGit::new(HEAD, true)
}

/// A report where every configured gate ran and passed.
fn green() -> GateReport {
    GateReport::new(["gates".to_owned()], Vec::new())
}

fn refusal(git: &FakeGit, plan: &Plan, policy: &ReleasePolicy, gates: &GateReport) -> String {
    permit(git, policy, plan, DIGEST, gates)
        .expect_err("the case must be refused")
        .key()
        .to_owned()
}

/// The documented case is permitted, and the permit carries what the tag will say.
#[rstest]
fn a_clean_repository_on_the_planned_commit_is_permitted() {
    let policy = ReleasePolicy::parse(POLICY).expect("policy");
    let permit = permit(&git(), &policy, &plan(), DIGEST, &green()).expect("permitted");

    assert_eq!(permit.tag(), "v0.2.0");
    assert_eq!(permit.commit(), HEAD);
}

/// AC#2: a dirty tree is refused before anything is staged.
#[rstest]
fn a_dirty_tree_is_refused() {
    let policy = ReleasePolicy::parse(POLICY).expect("policy");
    let mut git = git();
    git.set_clean(false);

    assert_eq!(refusal(&git, &plan(), &policy, &green()), "worktree");
}

/// AC#1, DoD#2: a change the plan never recorded refuses the commit, names the path, and says what
/// the operator can do about it.
#[rstest]
fn a_change_outside_the_recorded_set_is_refused_by_path_and_remedy() {
    let policy = ReleasePolicy::parse(POLICY).expect("policy");
    let mut git = git();
    git.set_dirty_paths([
        "CHANGELOG.md".to_owned(),
        "Cargo.toml".to_owned(),
        "notes.md".to_owned(),
    ]);

    let error = permit(&git, &policy, &plan(), DIGEST, &green()).expect_err("an unrecorded change");

    assert_eq!(error.key(), "worktree");
    assert!(error.detail().contains("notes.md"), "{}", error.detail());
    assert!(
        error.detail().contains("run prepare") && error.detail().contains("revert"),
        "{}",
        error.detail()
    );
}

/// AC#2: a tree with nothing left to stage is refused — `cut` is not a no-op.
#[rstest]
fn a_tree_with_nothing_to_stage_is_refused() {
    let policy = ReleasePolicy::parse(POLICY).expect("policy");

    let error =
        permit(&unprepared(), &policy, &plan(), DIGEST, &green()).expect_err("nothing to commit");

    assert_eq!(error.key(), "worktree");
    assert!(error.detail().contains("prepare"), "{}", error.detail());
}

/// AC#2: `cut` runs on a branch, never on a detached head.
#[rstest]
fn a_detached_head_is_refused() {
    let policy = ReleasePolicy::parse(POLICY).expect("policy");
    let mut git = git();
    git.set_branch(None);

    assert_eq!(refusal(&git, &plan(), &policy, &green()), "branch");
}

/// AC#2: a plan made on a dirty tree is not cuttable even when the tree is clean now.
#[rstest]
fn a_plan_derived_from_a_dirty_tree_is_refused() {
    let policy = ReleasePolicy::parse(POLICY).expect("policy");
    let dirty_plan = Plan::propose(
        &policy,
        PlanInputs {
            source: SourceRevision::new(HEAD, false).expect("revision"),
            policy_digest: DIGEST.to_owned(),
            tool_version: "0.1.0".to_owned(),
        },
        VersionAuthority::Explicit {
            version: Version::parse("0.2.0").expect("version"),
        },
    )
    .expect("plan")
    .with_touched(["Cargo.toml".to_owned()]);

    assert_eq!(
        refusal(&git(), &dirty_plan, &policy, &green()),
        "plan.source"
    );
}

/// AC#2: a stale plan — `HEAD` moved since it was derived — is refused.
#[rstest]
fn a_stale_plan_is_refused_when_head_moved() {
    let policy = ReleasePolicy::parse(POLICY).expect("policy");
    let mut git = git();
    git.set_head_sha("1111111111111111111111111111111111111111");

    assert_eq!(refusal(&git, &plan(), &policy, &green()), "plan.source");
}

/// AC#2: a policy edited after the plan was made is refused, not silently accepted.
#[rstest]
fn a_policy_edited_after_the_plan_is_refused() {
    let policy = ReleasePolicy::parse(POLICY).expect("policy");

    let error = permit(
        &git(),
        &policy,
        &plan(),
        "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        &green(),
    )
    .expect_err("the policy changed");

    assert_eq!(error.key(), "policy.digest");
}

/// A plan that records nothing would cap a commit with no content.
#[rstest]
fn a_plan_without_a_touched_set_is_refused() {
    let policy = ReleasePolicy::parse(POLICY).expect("policy");
    let bare = Plan::propose(
        &policy,
        PlanInputs {
            source: SourceRevision::new(HEAD, true).expect("revision"),
            policy_digest: DIGEST.to_owned(),
            tool_version: "0.1.0".to_owned(),
        },
        VersionAuthority::Explicit {
            version: Version::parse("0.2.0").expect("version"),
        },
    )
    .expect("plan");

    assert_eq!(refusal(&git(), &bare, &policy, &green()), "plan.touched");
}

/// AC#2: a tag that already points somewhere else is never moved.
#[rstest]
fn a_tag_pointing_at_another_commit_is_refused() {
    let policy = ReleasePolicy::parse(POLICY).expect("policy");
    let mut git = git();
    git.add_tag(TagRef::new(
        "v0.2.0",
        "2222222222222222222222222222222222222222",
    ));

    assert_eq!(refusal(&git, &plan(), &policy, &green()), "tag");
}

/// An existing tag at the planned commit means the release is already cut.
#[rstest]
fn a_tag_already_at_the_planned_commit_is_refused() {
    let policy = ReleasePolicy::parse(POLICY).expect("policy");
    let mut git = git();
    git.add_tag(TagRef::new("v0.2.0", HEAD));

    assert_eq!(refusal(&git, &plan(), &policy, &green()), "tag");
}

/// AC#2: a failing gate is refused, and the message names the verb.
#[rstest]
fn a_failed_gate_is_refused() {
    let policy = ReleasePolicy::parse(POLICY).expect("policy");
    let gates = GateReport::new(Vec::new(), ["gates".to_owned()]);

    let error = permit(&git(), &policy, &plan(), DIGEST, &gates).expect_err("gate failed");

    assert_eq!(error.key(), "gates.required");
    assert!(error.detail().contains("gates"), "{}", error.detail());
}

/// A configured gate that never ran is not a pass.
#[rstest]
fn a_gate_that_did_not_run_is_refused() {
    let policy = ReleasePolicy::parse(POLICY).expect("policy");
    let gates = GateReport::new(Vec::new(), Vec::new());

    let error = permit(&git(), &policy, &plan(), DIGEST, &gates).expect_err("gate missing");

    assert_eq!(error.key(), "gates.required");
}

/// AC#3: `cut` performs stage, commit and tag — and nothing that could reach a registry.
#[rstest]
fn cut_stages_commits_and_tags_without_publishing() {
    let policy = ReleasePolicy::parse(POLICY).expect("policy");
    let plan = plan();
    let permit: CutPermit = permit(&git(), &policy, &plan, DIGEST, &green()).expect("permitted");
    let mut git = git();

    let outcome = cut(&mut git, &plan, &permit, false).expect("cut succeeds");

    // The touched set is canonical, so the staged paths are sorted whatever order prepare saw.
    assert_eq!(
        git.operations(),
        &[
            GitOp::Stage(vec!["CHANGELOG.md".to_owned(), "Cargo.toml".to_owned()]),
            GitOp::Commit("chore(release): v0.2.0".to_owned()),
            GitOp::Tag("v0.2.0".to_owned()),
        ]
    );
    assert_eq!(outcome.tag(), "v0.2.0");
}

/// Requesting the push adds exactly one more operation, and still no publication.
#[rstest]
fn cut_pushes_only_when_asked() {
    let policy = ReleasePolicy::parse(POLICY).expect("policy");
    let plan = plan();
    let permit = permit(&git(), &policy, &plan, DIGEST, &green()).expect("permitted");
    let mut git = git();

    cut(&mut git, &plan, &permit, true).expect("cut succeeds");

    let operations = git.operations();
    assert!(
        matches!(operations.last(), Some(GitOp::Push(_))),
        "{operations:?}"
    );
    assert!(
        operations.iter().all(|op| matches!(
            op,
            GitOp::Stage(_) | GitOp::Commit(_) | GitOp::Tag(_) | GitOp::Push(_)
        )),
        "no operation reaches a registry"
    );
}

/// A Git failure surfaces as a failure, and the operation is not recorded as done.
#[rstest]
fn a_git_failure_is_reported() {
    let policy = ReleasePolicy::parse(POLICY).expect("policy");
    let plan = plan();
    let permit = permit(&git(), &policy, &plan, DIGEST, &green()).expect("permitted");
    let mut git = git();
    git.fail_next("index is locked");

    let error = cut(&mut git, &plan, &permit, false).expect_err("stage fails");

    assert!(error.to_string().contains("index is locked"), "{error}");
    assert!(git.operations().is_empty());
}

proptest! {
    /// The permit is exactly the conjunction of the refusals: whichever fact is wrong, it is the
    /// key that comes back.
    #[test]
    fn any_broken_fact_refuses_with_its_own_key(
        clean in any::<bool>(),
        on_branch in any::<bool>(),
        same_commit in any::<bool>(),
    ) {
        let policy = ReleasePolicy::parse(POLICY).expect("policy");
        let mut git = FakeGit::new(HEAD, true);
        git.set_dirty_paths(["CHANGELOG.md".to_owned(), "Cargo.toml".to_owned()]);
        if !clean {
            git.set_clean(false);
        }
        if !on_branch {
            git.set_branch(None);
        }
        if !same_commit {
            git.set_head_sha("3333333333333333333333333333333333333333");
        }

        let outcome = permit(&git, &policy, &plan(), DIGEST, &green());
        // The checks run in reporting order: branch, then the plan's commit, then the worktree.
        let expected = if !on_branch {
            Some("branch")
        } else if !same_commit {
            Some("plan.source")
        } else if !clean {
            Some("worktree")
        } else {
            None
        };

        match expected {
            Some(key) => {
                let refused = outcome.expect_err("refused");
                prop_assert_eq!(refused.key(), key);
            }
            None => prop_assert!(outcome.is_ok()),
        }
    }

    /// A tag name is the policy format with the version substituted, never the raw version.
    #[test]
    fn the_permitted_tag_matches_the_plan(
        major in 0u64..40,
        minor in 0u64..40,
        patch in 0u64..40,
    ) {
        let policy = ReleasePolicy::parse(POLICY).expect("policy");
        let version = Version::parse(&format!("{major}.{minor}.{patch}")).expect("semver");
        let plan = Plan::propose(
            &policy,
            PlanInputs {
                source: SourceRevision::new(HEAD, true).expect("revision"),
                policy_digest: DIGEST.to_owned(),
                tool_version: "0.1.0".to_owned(),
            },
            VersionAuthority::Explicit { version: version.clone() },
        )
        .expect("plan")
        .with_touched(["Cargo.toml".to_owned(), "CHANGELOG.md".to_owned()]);
        let permit = permit(&git(), &policy, &plan, DIGEST, &green()).expect("permitted");

        prop_assert_eq!(permit.tag(), plan.tag());
        prop_assert!(permit.tag().ends_with(version.as_str()));
    }
}
