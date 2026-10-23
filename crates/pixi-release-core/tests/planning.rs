use pixi_release_core::{ReleasePhase, can_transition};
use rstest::rstest;

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
