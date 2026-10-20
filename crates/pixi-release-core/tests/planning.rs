use pixi_release_core::{ReleasePhase, ReleasePlan, can_transition};
use rstest::{fixture, rstest};

#[fixture]
fn plan() -> ReleasePlan {
    ReleasePlan::new(
        "0.1.0",
        vec!["github-release".into(), "prefix".into(), "prefix".into()],
    )
}

#[rstest]
fn plan_evidence_is_deterministic(plan: ReleasePlan) {
    assert_eq!(plan.version(), "0.1.0");
    assert_eq!(plan.targets(), ["github-release", "prefix"]);
}

#[rstest]
#[case(ReleasePhase::Plan, ReleasePhase::Prepare, true)]
#[case(ReleasePhase::Verify, ReleasePhase::Publish, true)]
#[case(ReleasePhase::Plan, ReleasePhase::Publish, false)]
#[case(ReleasePhase::Cut, ReleasePhase::Publish, false)]
fn irreversible_steps_cannot_be_skipped(
    #[case] from: ReleasePhase,
    #[case] to: ReleasePhase,
    #[case] allowed: bool,
) {
    assert_eq!(can_transition(from, to), allowed);
}
