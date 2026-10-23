//! Release-domain types with no process, filesystem, or network dependency.

pub mod config;
pub mod plan;

/// The irreversible-ordering boundary for a release run.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ReleasePhase {
    /// Inspect policy and derive an immutable proposal.
    Plan,
    /// Make reviewed release changes without tagging or publishing.
    Prepare,
    /// Create the release commit and tag after gates pass.
    Cut,
    /// Prove tagged source and artifacts satisfy the plan.
    Verify,
    /// Upload verified artifacts to configured targets.
    Publish,
    /// Continue an interrupted publish from recorded target evidence.
    Resume,
}

/// States whether a phase may transition to the next requested phase.
#[must_use]
pub const fn can_transition(from: ReleasePhase, to: ReleasePhase) -> bool {
    matches!(
        (from, to),
        (ReleasePhase::Plan, ReleasePhase::Prepare)
            | (ReleasePhase::Prepare, ReleasePhase::Cut)
            | (ReleasePhase::Cut, ReleasePhase::Verify)
            | (ReleasePhase::Verify, ReleasePhase::Publish)
            | (ReleasePhase::Publish, ReleasePhase::Resume)
    )
}
