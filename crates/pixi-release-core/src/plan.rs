//! The immutable release plan: the one artifact every later phase re-checks.
//!
//! A plan is evidence, not a script. It names the version, the tag, the authority that chose the
//! version, the revision it was derived from, the policy digest it was built under, and the
//! targets it will publish to — and nothing in it depends on the order the policy happened to be
//! written in. `plan` itself performs no I/O (`doc-2` §2.3, REL-3); the caller reads the
//! repository and writes the file.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::config::{ConvcoEvidence, ReleasePolicy};

/// The plan schema this build emits.
pub const PLAN_SCHEMA: u32 = 1;

/// The tool recorded in every plan, independent of the crate's package name.
pub const TOOL_NAME: &str = "pixi-release";

/// How a version was chosen.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Bump {
    /// Let the authority decide from the commits since the last tag.
    Auto,
    /// Force a major bump.
    Major,
    /// Force a minor bump.
    Minor,
    /// Force a patch bump.
    Patch,
}

impl Bump {
    /// The value as it appears in a plan and on a command line.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Major => "major",
            Self::Minor => "minor",
            Self::Patch => "patch",
        }
    }

    /// Parses the command-line spelling.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "auto" => Some(Self::Auto),
            "major" => Some(Self::Major),
            "minor" => Some(Self::Minor),
            "patch" => Some(Self::Patch),
            _ => None,
        }
    }
}

impl fmt::Display for Bump {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A semantic version, validated once so every later phase can trust it.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Version(String);

impl Version {
    /// Parses `major.minor.patch` with an optional pre-release and build suffix.
    ///
    /// # Errors
    ///
    /// Refuses a leading `v`, a short triple, a leading zero, or an empty identifier.
    pub fn parse(candidate: &str) -> Result<Self, PlanError> {
        if candidate.is_empty() {
            return Err(PlanError::new("version", "a version is required"));
        }
        let (rest, build) = split_once(candidate, '+');
        let (core, pre_release) = split_once(rest, '-');
        let parts: Vec<&str> = core.split('.').collect();
        if parts.len() != 3 {
            return Err(PlanError::new(
                "version",
                format!("`{candidate}` is not `major.minor.patch`"),
            ));
        }
        for part in &parts {
            if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(PlanError::new(
                    "version",
                    format!("`{part}` in `{candidate}` is not a number"),
                ));
            }
            if part.len() > 1 && part.starts_with('0') {
                return Err(PlanError::new(
                    "version",
                    format!("`{part}` in `{candidate}` has a leading zero"),
                ));
            }
        }
        for suffix in [pre_release, build].into_iter().flatten() {
            if !valid_identifiers(suffix) {
                return Err(PlanError::new(
                    "version",
                    format!("`{suffix}` in `{candidate}` is not a valid identifier list"),
                ));
            }
        }
        Ok(Self(candidate.to_owned()))
    }

    /// The version as written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

fn split_once(value: &str, separator: char) -> (&str, Option<&str>) {
    value
        .split_once(separator)
        .map_or((value, None), |(head, tail)| (head, Some(tail)))
}

fn valid_identifiers(suffix: &str) -> bool {
    suffix.split('.').all(|identifier| {
        !identifier.is_empty()
            && identifier
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    })
}

/// The commit a plan was derived from, and whether the worktree was clean at the time.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct SourceRevision {
    commit: String,
    clean: bool,
}

impl SourceRevision {
    /// Records a revision.
    ///
    /// # Errors
    ///
    /// Refuses anything that is not a 7-to-40 character hexadecimal commit id.
    pub fn new(commit: impl Into<String>, clean: bool) -> Result<Self, PlanError> {
        let commit = commit.into();
        if !(7..=40).contains(&commit.len()) || !commit.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(PlanError::new(
                "source.commit",
                format!("`{commit}` is not a commit id"),
            ));
        }
        Ok(Self {
            commit: commit.to_lowercase(),
            clean,
        })
    }

    /// The commit id.
    #[must_use]
    pub fn commit(&self) -> &str {
        &self.commit
    }

    /// Whether the worktree was clean when the plan was derived.
    #[must_use]
    pub const fn is_clean(&self) -> bool {
        self.clean
    }
}

/// The digest of the authored policy a plan was built under (`decision-3`).
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct PolicyDigest(String);

impl PolicyDigest {
    /// Validates `sha256:` followed by 64 lowercase hexadecimal characters.
    ///
    /// # Errors
    ///
    /// Refuses a bare digest, another algorithm, or a truncated value, so a plan can never carry
    /// something that is not comparable to the digest in a rendered file.
    pub fn new(candidate: impl Into<String>) -> Result<Self, PlanError> {
        let candidate = candidate.into();
        let hex = candidate.strip_prefix("sha256:").unwrap_or_default();
        if hex.len() != 64
            || !hex
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
        {
            return Err(PlanError::new(
                "policy.digest",
                format!("`{candidate}` is not a sha256 digest"),
            ));
        }
        Ok(Self(candidate))
    }

    /// The digest as written, algorithm prefix included.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One publication target in a plan.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct TargetPlan {
    name: String,
    required: bool,
}

impl TargetPlan {
    /// Records a target.
    #[must_use]
    pub fn new(name: impl Into<String>, required: bool) -> Self {
        Self {
            name: name.into(),
            required,
        }
    }

    /// The registry target name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether publication to this target must succeed for the release to count.
    #[must_use]
    pub const fn required(&self) -> bool {
        self.required
    }
}

/// Canonicalises target rows: one row per name, sorted, with conflicting duplicates refused.
///
/// # Errors
///
/// Returns the first name declared both required and optional: silently picking one would make
/// the plan depend on the order the policy was written in.
pub fn normalize_targets<I>(targets: I) -> Result<Vec<TargetPlan>, PlanError>
where
    I: IntoIterator<Item = (String, bool)>,
{
    let mut rows: Vec<TargetPlan> = targets
        .into_iter()
        .map(|(name, required)| TargetPlan::new(name, required))
        .collect();
    rows.sort_by(|left, right| left.name.cmp(&right.name));

    let mut normalized: Vec<TargetPlan> = Vec::with_capacity(rows.len());
    for row in rows {
        match normalized.last() {
            Some(previous) if previous.name == row.name => {
                if previous.required != row.required {
                    return Err(PlanError::new(
                        "target.name",
                        format!("`{}` is both required and optional", row.name),
                    ));
                }
            }
            _ => normalized.push(row),
        }
    }
    Ok(normalized)
}

/// Where the planned version came from.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VersionAuthority {
    /// A human named the version; the plan records the request, not a derivation.
    Explicit {
        /// The requested version.
        version: Version,
    },
    /// Convco derived the version from the commits, and its evidence travels with the plan.
    Convco {
        /// The bump that was requested.
        bump: Bump,
        /// The version Convco returned.
        version: Version,
        /// The tool version and configuration digests Convco read (`decision-6`).
        evidence: ConvcoEvidence,
    },
}

/// The repository facts a plan records, supplied by the caller so `plan` stays pure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanInputs {
    /// The commit the plan was derived from.
    pub source: SourceRevision,
    /// `sha256:` digest of the policy file the plan was built under.
    pub policy_digest: String,
    /// The version of the tool that produced the plan.
    pub tool_version: String,
}

/// A canonical, reviewable release proposal.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Plan {
    schema: u32,
    version: Version,
    tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    bump: Option<Bump>,
    authority: Authority,
    source: SourceRevision,
    policy: PolicyRef,
    targets: Vec<TargetPlan>,
    /// Paths `prepare` reported it would change, recorded back into the plan (`doc-3` §5.7).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    touched: Vec<String>,
    tool: Tool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum Authority {
    Convco {
        version: String,
        config: BTreeMap<String, String>,
    },
    Explicit {
        requested: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
struct PolicyRef {
    digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
struct Tool {
    name: String,
    version: String,
}

impl Plan {
    /// Proposes the plan for `policy` under the supplied repository facts.
    ///
    /// # Errors
    ///
    /// Refuses a Convco derivation under a manifest policy, a policy that admits no target, a
    /// malformed digest, and an empty tool version — each naming the key at fault.
    pub fn propose(
        policy: &ReleasePolicy,
        inputs: PlanInputs,
        authority: VersionAuthority,
    ) -> Result<Self, PlanError> {
        let digest = PolicyDigest::new(inputs.policy_digest)?;
        if inputs.tool_version.trim().is_empty() {
            return Err(PlanError::new("tool.version", "a plan names its author"));
        }

        let (version, bump, authority) = match authority {
            VersionAuthority::Explicit { version } => {
                let requested = version.to_string();
                (version, None, Authority::Explicit { requested })
            }
            VersionAuthority::Convco {
                bump,
                version,
                evidence,
            } => {
                // One rule, one place: the policy decides whether Convco evidence is admissible
                // here, and its refusal already cites `decision-6`.
                policy
                    .plan_evidence(Some(evidence.clone()))
                    .map_err(|error| PlanError::new(error.key(), error.detail()))?;
                let authority = Authority::Convco {
                    version: evidence.version().to_owned(),
                    config: evidence.config_hashes().clone(),
                };
                (version, Some(bump), authority)
            }
        };

        let targets = normalize_targets(
            policy
                .targets()
                .iter()
                .map(|target| (target.name().to_owned(), target.required())),
        )?;
        if targets.is_empty() {
            return Err(PlanError::new(
                "target.name",
                "a plan with no target would publish nowhere",
            ));
        }

        let tag = policy.tag_format().replace("{version}", version.as_str());
        Ok(Self {
            schema: PLAN_SCHEMA,
            version,
            tag,
            bump,
            authority,
            source: inputs.source,
            policy: PolicyRef { digest: digest.0 },
            targets,
            touched: Vec::new(),
            tool: Tool {
                name: TOOL_NAME.to_owned(),
                version: inputs.tool_version,
            },
        })
    }

    /// Reads a plan back from its canonical JSON, re-validating every checked field.
    ///
    /// # Errors
    ///
    /// Refuses JSON that is not a plan, and a version, revision or digest that would not have been
    /// accepted when the plan was proposed — a plan file is evidence, not a trusted input.
    pub fn from_json(text: &str) -> Result<Self, PlanError> {
        let plan: Self = serde_json::from_str(text)
            .map_err(|error| PlanError::new("plan", error.to_string()))?;
        let version = Version::parse(plan.version.as_str())?;
        let source = SourceRevision::new(plan.source.commit.clone(), plan.source.clean)?;
        PolicyDigest::new(plan.policy.digest.clone())?;
        if plan.tool.version.trim().is_empty() {
            return Err(PlanError::new("tool.version", "a plan names its author"));
        }
        if plan.tag.is_empty() {
            return Err(PlanError::new(
                "tag",
                "a plan carries the tag it will create",
            ));
        }
        Ok(Self {
            version,
            source,
            ..plan
        })
    }

    /// The planned version.
    #[must_use]
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// The tag the plan will be anchored to (`decision-4`).
    #[must_use]
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// The bump that was requested, when Convco derived the version.
    #[must_use]
    pub const fn bump(&self) -> Option<Bump> {
        self.bump
    }

    /// Canonical target rows, sorted with one row per name.
    #[must_use]
    pub fn targets(&self) -> &[TargetPlan] {
        &self.targets
    }

    /// Paths the plan records as the only ones `cut` may stage.
    #[must_use]
    pub fn touched(&self) -> &[String] {
        &self.touched
    }

    /// Records the touched set `prepare` reported, sorted and deduplicated.
    ///
    /// The plan is immutable once cut, so this consumes and returns: a caller cannot quietly edit
    /// the evidence a later phase already holds.
    #[must_use]
    pub fn with_touched<I>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        let mut touched: Vec<String> = paths.into_iter().collect();
        touched.sort();
        touched.dedup();
        self.touched = touched;
        self
    }

    /// The tool version that produced the plan.
    #[must_use]
    pub fn tool_version(&self) -> &str {
        &self.tool.version
    }

    /// The `sha256:` digest of this plan's canonical JSON, written into the tag (`decision-4`).
    #[must_use]
    pub fn digest(&self) -> String {
        use sha2::{Digest as _, Sha256};
        let digest = Sha256::digest(self.to_json().as_bytes());
        let hex: String = digest
            .iter()
            .fold(String::with_capacity(64), |mut acc, byte| {
                use std::fmt::Write as _;
                let _ = write!(acc, "{byte:02x}");
                acc
            });
        format!("sha256:{hex}")
    }

    /// The revision the plan was derived from.
    #[must_use]
    pub fn source(&self) -> &SourceRevision {
        &self.source
    }

    /// The policy digest the plan was built under.
    #[must_use]
    pub fn policy_digest(&self) -> &str {
        &self.policy.digest
    }

    /// Renders the plan as canonical JSON, ending in exactly one newline.
    ///
    /// # Panics
    ///
    /// Never in practice: every field is a string, a boolean, or a sequence of them.
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut rendered =
            serde_json::to_string_pretty(self).expect("a plan is a tree of serialisable fields");
        rendered.push('\n');
        rendered
    }
}

/// A refusal to build a plan, naming the key at fault.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanError {
    key: String,
    detail: String,
}

impl PlanError {
    /// Builds a refusal for `key` with an explanation.
    #[must_use]
    pub fn new(key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            detail: detail.into(),
        }
    }

    /// The dotted key or field at fault.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The explanation.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for PlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.key, self.detail)
    }
}

impl std::error::Error for PlanError {}
