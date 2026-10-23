//! Semantic Git boundary: the read facts a plan records, plus the mutations `cut` is allowed.
//!
//! Two traits, deliberately unequal. [`GitProtocol`] only observes, so a plan and every refusal can
//! be proven against [`FakeGit`] without a repository. [`ReleaseGit`] adds the three mutations
//! `cut` performs — stage, commit, tag — and nothing else: publication has no port here, which is
//! how `cut` cannot publish (`doc-2` §7.6).

pub mod local;

pub use local::LocalGit;

use std::fmt;

/// A local tag and the commit it points at.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TagRef {
    name: String,
    commit: String,
}

impl TagRef {
    /// Records a tag.
    #[must_use]
    pub fn new(name: impl Into<String>, commit: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            commit: commit.into(),
        }
    }

    /// The tag name, for example `v0.2.0`.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The commit the tag resolves to.
    #[must_use]
    pub fn commit(&self) -> &str {
        &self.commit
    }
}

/// The observable repository facts.
pub trait GitProtocol {
    /// The commit `HEAD` points at.
    fn head_sha(&self) -> &str;

    /// Whether the worktree has no uncommitted changes.
    fn is_clean(&self) -> bool {
        self.dirty_paths().is_empty()
    }

    /// The paths with uncommitted changes, so `cut` can prove they are the recorded ones.
    fn dirty_paths(&self) -> &[String];

    /// The checked-out branch, or `None` when `HEAD` is detached.
    fn branch(&self) -> Option<&str>;

    /// Local tags, in the order the adapter reports them.
    fn tags(&self) -> &[TagRef];
}

/// One recorded mutation, so a test can assert exactly what happened.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GitOp {
    /// `git add -- <paths>`.
    Stage(Vec<String>),
    /// `git commit -m <message>`.
    Commit(String),
    /// `git tag -a <name> -m <message>`.
    Tag(String),
    /// `git push <refspec>`.
    Push(String),
}

/// Why a Git mutation could not be performed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitError {
    operation: String,
    detail: String,
}

impl GitError {
    /// Builds an error naming the operation that failed.
    #[must_use]
    pub fn new(operation: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            detail: detail.into(),
        }
    }

    /// The operation that failed.
    #[must_use]
    pub fn operation(&self) -> &str {
        &self.operation
    }

    /// What the adapter reported.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for GitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.operation, self.detail)
    }
}

impl std::error::Error for GitError {}

/// The mutations `cut` may perform.
pub trait ReleaseGit: GitProtocol {
    /// Stages exactly the paths the plan recorded.
    ///
    /// # Errors
    ///
    /// Returns the adapter's failure.
    fn stage(&mut self, paths: &[String]) -> Result<(), GitError>;

    /// Creates the release commit and returns its id.
    ///
    /// # Errors
    ///
    /// Returns the adapter's failure.
    fn commit(&mut self, message: &str) -> Result<String, GitError>;

    /// Creates the annotated release tag.
    ///
    /// # Errors
    ///
    /// Returns the adapter's failure.
    fn tag(&mut self, name: &str, message: &str) -> Result<(), GitError>;

    /// Pushes `refspec` to the configured remote.
    ///
    /// # Errors
    ///
    /// Returns the adapter's failure.
    fn push(&mut self, refspec: &str) -> Result<(), GitError>;

    /// The mutations performed so far, in order.
    fn operations(&self) -> &[GitOp];
}

/// Stateful in-memory adapter used to test release policy without a subprocess.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FakeGit {
    head_sha: String,
    dirty_paths: Vec<String>,
    branch: Option<String>,
    tags: Vec<TagRef>,
    operations: Vec<GitOp>,
    fail_next: Option<String>,
}

impl FakeGit {
    /// Creates fake repository state on the `main` branch with no tags.
    #[must_use]
    pub fn new(head_sha: impl Into<String>, clean: bool) -> Self {
        Self {
            head_sha: head_sha.into(),
            dirty_paths: if clean {
                Vec::new()
            } else {
                vec!["src/main.rs".to_owned()]
            },
            branch: Some("main".to_owned()),
            tags: Vec::new(),
            operations: Vec::new(),
            fail_next: None,
        }
    }

    /// Changes whether the fake worktree is clean, with one unaccounted change when dirty.
    pub fn set_clean(&mut self, clean: bool) {
        self.dirty_paths = if clean {
            Vec::new()
        } else {
            vec!["src/main.rs".to_owned()]
        };
    }

    /// Sets the exact paths with uncommitted changes.
    pub fn set_dirty_paths<I>(&mut self, paths: I)
    where
        I: IntoIterator<Item = String>,
    {
        self.dirty_paths = paths.into_iter().collect();
    }

    /// Sets the checked-out branch; `None` detaches `HEAD`.
    pub fn set_branch(&mut self, branch: Option<&str>) {
        self.branch = branch.map(ToOwned::to_owned);
    }

    /// Moves `HEAD` to another commit, as an intervening commit would.
    pub fn set_head_sha(&mut self, head_sha: impl Into<String>) {
        self.head_sha = head_sha.into();
    }

    /// Adds a local tag.
    pub fn add_tag(&mut self, tag: TagRef) {
        self.tags.push(tag);
    }

    /// Makes the next mutation fail with `detail`, leaving the operation unrecorded.
    pub fn fail_next(&mut self, detail: impl Into<String>) {
        self.fail_next = Some(detail.into());
    }
}

impl GitProtocol for FakeGit {
    fn head_sha(&self) -> &str {
        &self.head_sha
    }

    fn dirty_paths(&self) -> &[String] {
        &self.dirty_paths
    }

    fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }

    fn tags(&self) -> &[TagRef] {
        &self.tags
    }
}

impl FakeGit {
    /// Records `operation`, or reports the injected failure.
    fn record(&mut self, operation: GitOp) -> Result<(), GitError> {
        if let Some(detail) = self.fail_next.take() {
            return Err(GitError::new("fake", detail));
        }
        self.operations.push(operation);
        Ok(())
    }
}

impl ReleaseGit for FakeGit {
    fn stage(&mut self, paths: &[String]) -> Result<(), GitError> {
        self.record(GitOp::Stage(paths.to_vec()))
    }

    fn commit(&mut self, message: &str) -> Result<String, GitError> {
        self.record(GitOp::Commit(message.to_owned()))?;
        Ok(format!(
            "{}0000000000000000000000000000000000000000",
            &self.head_sha[..7]
        ))
    }

    fn tag(&mut self, name: &str, _message: &str) -> Result<(), GitError> {
        self.record(GitOp::Tag(name.to_owned()))
    }

    fn push(&mut self, refspec: &str) -> Result<(), GitError> {
        self.record(GitOp::Push(refspec.to_owned()))
    }

    fn operations(&self) -> &[GitOp] {
        &self.operations
    }
}
