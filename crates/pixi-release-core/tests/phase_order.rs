//! rstest-bdd scenarios for the release phase order.

use pixi_release_core::{ReleasePhase, can_transition};
use rstest_bdd_macros::{scenario, then};

fn parse_phase(name: &str) -> ReleasePhase {
    match name {
        "plan" => ReleasePhase::Plan,
        "prepare" => ReleasePhase::Prepare,
        "cut" => ReleasePhase::Cut,
        "verify" => ReleasePhase::Verify,
        "publish" => ReleasePhase::Publish,
        "resume" => ReleasePhase::Resume,
        other => panic!("unknown release phase: {other}"),
    }
}

#[then("{from} may move to {to}: {allowed}")]
fn transition_is_allowed(from: String, to: String, allowed: bool) {
    assert_eq!(
        can_transition(parse_phase(&from), parse_phase(&to)),
        allowed
    );
}

// rstest-bdd injects one case per Examples row, so the scenario function declares one
// parameter per column. The body re-checks the row against the same rule the step checks
// against the substituted text: the step proves the Gherkin binding, the body proves the
// table itself.
#[scenario(path = "tests/features/phase_order.feature")]
fn phase_order(from: String, to: String, allowed: String) {
    let expected: bool = allowed.parse().expect("allowed column is a boolean");
    assert_eq!(
        can_transition(parse_phase(&from), parse_phase(&to)),
        expected
    );
}
