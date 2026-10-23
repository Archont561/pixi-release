//! The two irreversible operations, and the proof required before them.
//!
//! [`permit`] is the gate: it reads the repository through [`GitProtocol`] and returns a
//! [`CutPermit`] only when every refusal is answered. [`cut`] will not run without that permit, so
//! "validated before anything moved" is a type, not a convention. Neither function reaches a
//! publication target: `ReleaseGit` has no such operation to call (`doc-2` §7.6).

use std::fmt;

use pixi_release_git::{GitError, ReleaseGit};

use crate::plan::Plan;

/// What the configured gates reported.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GateReport {
    passed: Vec<String>,
    failed: Vec<String>,
}

impl GateReport {
    /// Records which verbs passed and which failed.
    #[must_use]
    pub fn new<I, J>(passed: I, failed: J) -> Self
    where
        I: IntoIterator<Item = String>,
        J: IntoIterator<Item = String>,
    {
        Self {
            passed: passed.into_iter().collect(),
            failed: failed.into_iter().collect(),
        }
    }

    /// The verbs that ran and passed.
    #[must_use]
    pub fn passed(&self) -> &[String] {
        &self.passed
    }

    /// The verbs that ran and failed.
    #[must_use]
    pub fn failed(&self) -> &[String] {
        &self.failed
    }
}

/// A refusal to cut, naming the fact that is not satisfied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CutRefusal {
    key: String,
    detail: String,
}

impl CutRefusal {
    fn new(key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            detail: detail.into(),
        }
    }

    /// The key at fault, for example `worktree` or `plan.source`.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Why the cut was refused.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for CutRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.key, self.detail)
    }
}

impl std::error::Error for CutRefusal {}

/// Proof that every precondition held, and the three facts the tag will carry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CutPermit {
    tag: String,
    commit: String,
    branch: String,
}

impl CutPermit {
    /// The tag to create.
    #[must_use]
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// The commit being tagged.
    #[must_use]
    pub fn commit(&self) -> &str {
        &self.commit
    }

    /// The branch the cut will move when pushed.
    #[must_use]
    pub fn branch(&self) -> &str {
        &self.branch
    }
}

/// Checks every precondition, in the order a human would want them reported.
///
/// # Errors
///
/// Returns the first refusal: a dirty tree, a detached head, a plan derived from a dirty tree, a
/// stale plan whose commit is no longer `HEAD`, a policy edited since the plan, a plan that
/// recorded no touched set, a tag that already exists, or a gate that failed or never ran.
pub fn permit<G>(
    git: &G,
    policy: &crate::config::ReleasePolicy,
    plan: &Plan,
    current_policy_digest: &str,
    gates: &GateReport,
) -> Result<CutPermit, CutRefusal>
where
    G: pixi_release_git::GitProtocol + ?Sized,
{
    let (tag, commit, branch) = check_repository(git, policy, plan, current_policy_digest)?;
    if let Some(failed) = gates.failed().first() {
        return Err(CutRefusal::new(
            "gates.required",
            format!("`{failed}` failed, so the release is not proven"),
        ));
    }
    for verb in policy.gates() {
        if !gates.passed().iter().any(|passed| passed == verb) {
            return Err(CutRefusal::new(
                "gates.required",
                format!("`{verb}` is required by the policy but did not run"),
            ));
        }
    }
    Ok(CutPermit {
        tag,
        commit,
        branch,
    })
}

/// Everything the repository alone can answer, so a caller can refuse before running gates.
///
/// # Errors
///
/// Returns the first refusal that does not depend on gate results.
pub fn check_repository<G>(
    git: &G,
    policy: &crate::config::ReleasePolicy,
    plan: &Plan,
    current_policy_digest: &str,
) -> Result<(String, String, String), CutRefusal>
where
    G: pixi_release_git::GitProtocol + ?Sized,
{
    let Some(branch) = git.branch() else {
        return Err(CutRefusal::new(
            "branch",
            "HEAD is detached; cut runs on the branch the release commit belongs to",
        ));
    };
    if !plan.source().is_clean() {
        return Err(CutRefusal::new(
            "plan.source",
            "the plan was derived from a dirty tree, so it describes no reviewable commit",
        ));
    }
    if plan.source().commit() != git.head_sha() {
        return Err(CutRefusal::new(
            "plan.source",
            format!(
                "the plan describes {}, but HEAD is {}; re-plan",
                plan.source().commit(),
                git.head_sha()
            ),
        ));
    }
    if plan.policy_digest() != current_policy_digest {
        return Err(CutRefusal::new(
            "policy.digest",
            "the policy changed after the plan was made; re-plan",
        ));
    }
    if plan.touched().is_empty() {
        return Err(CutRefusal::new(
            "plan.touched",
            "the plan records no touched set, so cut would commit nothing; run prepare",
        ));
    }
    // `prepare` leaves the worktree dirty by design, so the rule is not "clean" — it is "nothing
    // outside the recorded set", which is what makes the commit reviewable (doc-2 §7.7).
    if let Some(unaccounted) = git
        .dirty_paths()
        .iter()
        .find(|path| !plan.touched().contains(path))
    {
        return Err(CutRefusal::new(
            "worktree",
            format!("`{unaccounted}` changed but the plan did not record it"),
        ));
    }
    if git.dirty_paths().is_empty() {
        return Err(CutRefusal::new(
            "worktree",
            "nothing is left to stage; run prepare before cut",
        ));
    }
    let rendered = policy
        .tag_format()
        .replace("{version}", plan.version().as_str());
    if rendered != plan.tag() {
        return Err(CutRefusal::new(
            "tag",
            format!(
                "the policy spells the tag `{rendered}`, but the plan carries `{}`; re-plan",
                plan.tag()
            ),
        ));
    }
    if let Some(existing) = git.tags().iter().find(|tag| tag.name() == plan.tag()) {
        let detail = if existing.commit() == plan.source().commit() {
            format!(
                "{} already points at this commit; the release is cut",
                plan.tag()
            )
        } else {
            format!(
                "{} already points at {}; refusing to move a tag",
                plan.tag(),
                existing.commit()
            )
        };
        return Err(CutRefusal::new("tag", detail));
    }
    Ok((
        plan.tag().to_owned(),
        plan.source().commit().to_owned(),
        branch.to_owned(),
    ))
}

/// What the cut did, for the caller to report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CutOutcome {
    commit: String,
    tag: String,
    pushed: bool,
}

impl CutOutcome {
    /// The release commit.
    #[must_use]
    pub fn commit(&self) -> &str {
        &self.commit
    }

    /// The tag that was created.
    #[must_use]
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Whether the branch and tag were pushed.
    #[must_use]
    pub const fn pushed(&self) -> bool {
        self.pushed
    }
}

/// Why a cut could not complete.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CutError {
    error: GitError,
}

impl fmt::Display for CutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.error)
    }
}

impl std::error::Error for CutError {}

/// The commit message a cut writes.
#[must_use]
pub fn commit_message(tag: &str) -> String {
    format!("chore(release): {tag}")
}

/// The annotated tag body: the plan's own evidence, so `verify` can re-derive it (`decision-4`).
#[must_use]
pub fn tag_message(plan: &Plan) -> String {
    format!(
        "{}\n\npixi-release-version: {}\nplan-digest: {}\npolicy-digest: {}",
        plan.tag(),
        plan.tool_version(),
        plan.digest(),
        plan.policy_digest()
    )
}

/// Performs the cut the permit allows: stage exactly the recorded set, commit, tag, optionally push.
///
/// # Errors
///
/// Returns the adapter's failure, with the operations already performed left in place for
/// inspection — a half-cut is visible rather than silent.
pub fn cut<R>(
    git: &mut R,
    plan: &Plan,
    permit: &CutPermit,
    push: bool,
) -> Result<CutOutcome, CutError>
where
    R: ReleaseGit + ?Sized,
{
    let wrap = |error: GitError| CutError { error };

    git.stage(plan.touched()).map_err(wrap)?;
    let commit = git.commit(&commit_message(permit.tag())).map_err(wrap)?;
    git.tag(permit.tag(), &tag_message(plan)).map_err(wrap)?;

    if push {
        git.push(&format!("refs/heads/{}", permit.branch()))
            .map_err(wrap)?;
        git.push(&format!("refs/tags/{}", permit.tag()))
            .map_err(wrap)?;
    }

    Ok(CutOutcome {
        commit,
        tag: permit.tag().to_owned(),
        pushed: push,
    })
}
