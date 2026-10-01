//! Semantic Git boundary. Shell and real-repository adapters will implement this port.

/// The smallest Git view release policy needs before it can cut a tag.
pub trait GitProtocol {
    /// Returns the commit selected for release evidence.
    fn head_sha(&self) -> &str;

    /// Returns whether there are no uncommitted changes.
    fn is_clean(&self) -> bool;
}

/// Stateful in-memory Git adapter used to test release policy without a subprocess.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FakeGit {
    head_sha: String,
    clean: bool,
}

impl FakeGit {
    /// Creates fake repository state.
    #[must_use]
    pub fn new(head_sha: impl Into<String>, clean: bool) -> Self {
        Self {
            head_sha: head_sha.into(),
            clean,
        }
    }

    /// Changes whether the fake worktree is clean.
    pub fn set_clean(&mut self, clean: bool) {
        self.clean = clean;
    }
}

impl GitProtocol for FakeGit {
    fn head_sha(&self) -> &str {
        &self.head_sha
    }

    fn is_clean(&self) -> bool {
        self.clean
    }
}
