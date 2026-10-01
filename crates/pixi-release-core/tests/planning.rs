use pixi_release_core::{ReleasePhase, ReleasePlan, can_transition};

#[test]
fn plan_evidence_is_deterministic() {
    let plan = ReleasePlan::new(
        "0.1.0",
        vec!["github-release".into(), "prefix".into(), "prefix".into()],
    );

    assert_eq!(plan.version(), "0.1.0");
    assert_eq!(plan.targets(), ["github-release", "prefix"]);
}

#[test]
fn irreversible_steps_cannot_be_skipped() {
    assert!(can_transition(ReleasePhase::Plan, ReleasePhase::Prepare));
    assert!(can_transition(ReleasePhase::Verify, ReleasePhase::Publish));
    assert!(!can_transition(ReleasePhase::Plan, ReleasePhase::Publish));
    assert!(!can_transition(ReleasePhase::Cut, ReleasePhase::Publish));
}
