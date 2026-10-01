//! Release-domain types with no process, filesystem, or network dependency.

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

/// A deterministic, reviewable release proposal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleasePlan {
    version: String,
    targets: Vec<String>,
}

impl ReleasePlan {
    /// Constructs a plan and normalises target order so identical input has identical evidence.
    #[must_use]
    pub fn new(version: impl Into<String>, mut targets: Vec<String>) -> Self {
        targets.sort();
        targets.dedup();
        Self {
            version: version.into(),
            targets,
        }
    }

    /// The proposed release version.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Publication targets in canonical order.
    #[must_use]
    pub fn targets(&self) -> &[String] {
        &self.targets
    }
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
