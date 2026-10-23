//! Typed release policy and its validation (`doc-3` §4).
//!
//! Parsing and every rule below are deterministic and perform no I/O, so a policy can be refused
//! before any release command touches the filesystem or Git. The document itself is the
//! canonical schema description; this module implements it rather than restating it.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::Deserialize;

/// The only schema this build understands.
pub const SUPPORTED_SCHEMA: u32 = 1;

/// The only target admissible before REL-6 (`decision-9`).
pub const ADMISSIBLE_TARGET: &str = "github-release";

/// The placeholder a tag format must carry (`decision-4`).
pub const TAG_PLACEHOLDER: &str = "{version}";

/// Workflows the renderer may own. `ci.yml` and `docs.yml` stay hand-written (`decision-10` §2).
const RENDERABLE: [&str; 2] = ["auto-release", "release"];

/// Inputs about the surrounding repository that change what is safe to accept.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PolicyContext {
    /// Whether a Convco configuration (`.versionrc` or `.convco`) is present.
    pub convco_configured: bool,
}

/// Where the release version comes from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VersionSource {
    /// Convco derives the version from conventional commits (`decision-6`).
    Convco,
    /// The version literal in the manifests is the source.
    Manifest,
}

/// One refusal, naming the exact key that is wrong.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyError {
    key: String,
    detail: String,
}

impl PolicyError {
    /// Builds a refusal for `key` with an explanation.
    #[must_use]
    pub fn new(key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            detail: detail.into(),
        }
    }

    /// The dotted policy key at fault, for example `project.version-source`.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Why the value is refused.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for PolicyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.key, self.detail)
    }
}

/// Every refusal produced by one parse, so an author fixes the document in one pass.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyErrors(Vec<PolicyError>);

impl PolicyErrors {
    fn one(error: PolicyError) -> Self {
        Self(vec![error])
    }

    /// The first named key, which is the shallowest fault.
    #[must_use]
    pub fn first_key(&self) -> &str {
        self.0.first().map_or("", PolicyError::key)
    }

    /// Iterates the refusals in document order.
    pub fn iter(&self) -> impl Iterator<Item = &PolicyError> {
        self.0.iter()
    }

    /// Number of refusals.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether there are no refusals. Always `false` for a value that exists.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for PolicyErrors {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, error) in self.0.iter().enumerate() {
            if index > 0 {
                formatter.write_str("; ")?;
            }
            write!(formatter, "{error}")?;
        }
        Ok(())
    }
}

/// One row of the build matrix. Policy data only: the controller never compiles (`decision-2`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildRow {
    platform: String,
    runner: String,
    args: Option<String>,
    artifacts: Vec<String>,
}

impl BuildRow {
    /// Pixi platform name, for example `linux-64`.
    #[must_use]
    pub fn platform(&self) -> &str {
        &self.platform
    }

    /// Runner label the workflow uses.
    #[must_use]
    pub fn runner(&self) -> &str {
        &self.runner
    }

    /// Argument string copied into the workflow verbatim.
    #[must_use]
    pub fn args(&self) -> Option<&str> {
        self.args.as_deref()
    }

    /// Artifact globs this row must produce.
    #[must_use]
    pub fn artifacts(&self) -> &[String] {
        &self.artifacts
    }
}

/// What to build, where, and what must come back.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildPolicy {
    task: String,
    environment: String,
    matrix: Vec<BuildRow>,
}

impl BuildPolicy {
    /// The repository task that performs the build.
    #[must_use]
    pub fn task(&self) -> &str {
        &self.task
    }

    /// The Pixi environment the task runs in.
    #[must_use]
    pub fn environment(&self) -> &str {
        &self.environment
    }

    /// One entry per platform.
    #[must_use]
    pub fn matrix(&self) -> &[BuildRow] {
        &self.matrix
    }
}

/// The clean-room proof settings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofPolicy {
    enabled: bool,
    isolated: bool,
    smoke: Vec<String>,
    expect_version: bool,
}

impl ProofPolicy {
    /// Whether the proof runs at all.
    #[must_use]
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Whether the proof runs with an empty environment and scratch `HOME`.
    #[must_use]
    pub fn isolated(&self) -> bool {
        self.isolated
    }

    /// Smoke commands run in the clean room.
    #[must_use]
    pub fn smoke(&self) -> &[String] {
        &self.smoke
    }

    /// Whether the binary's reported version must equal the tag.
    #[must_use]
    pub fn expect_version(&self) -> bool {
        self.expect_version
    }
}

/// Which workflows the renderer owns.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowsPolicy {
    render: Vec<String>,
    runner: Option<String>,
}

impl WorkflowsPolicy {
    /// Workflow names, in document order.
    #[must_use]
    pub fn render(&self) -> &[String] {
        &self.render
    }

    /// Runner label shared by rendered workflows.
    #[must_use]
    pub fn runner(&self) -> Option<&str> {
        self.runner.as_deref()
    }
}

/// One publication target. A target is a registry, not a `(registry, platform)` pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetPolicy {
    name: String,
    required: bool,
    channel: Option<String>,
    auth: Option<String>,
    platforms: Vec<String>,
}

impl TargetPolicy {
    /// Registry name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether a failure of this target makes the release incomplete.
    #[must_use]
    pub fn required(&self) -> bool {
        self.required
    }

    /// Channel or namespace, when the registry has one.
    #[must_use]
    pub fn channel(&self) -> Option<&str> {
        self.channel.as_deref()
    }

    /// Authentication mode, for example `oidc`.
    #[must_use]
    pub fn auth(&self) -> Option<&str> {
        self.auth.as_deref()
    }

    /// Platforms this target serves.
    #[must_use]
    pub fn platforms(&self) -> &[String] {
        &self.platforms
    }
}

/// A validated release policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleasePolicy {
    schema: u32,
    version_source: VersionSource,
    tag_format: String,
    bot_identity: Option<String>,
    manifests: Vec<String>,
    lockfiles: Vec<String>,
    changelog: String,
    gates: Vec<String>,
    build: Option<BuildPolicy>,
    publish_environment: Option<String>,
    proof: Option<ProofPolicy>,
    workflows: Option<WorkflowsPolicy>,
    targets: Vec<TargetPolicy>,
}

impl ReleasePolicy {
    /// Parses `text` with no Convco configuration assumed.
    ///
    /// # Errors
    ///
    /// Returns the same refusals as [`ReleasePolicy::parse_with`].
    pub fn parse(text: &str) -> Result<Self, PolicyErrors> {
        Self::parse_with(text, &PolicyContext::default())
    }

    /// Parses `text` and applies every rule that depends on `context`.
    ///
    /// # Errors
    ///
    /// Returns one refusal per fault, each naming the dotted key at fault.
    pub fn parse_with(text: &str, context: &PolicyContext) -> Result<Self, PolicyErrors> {
        let raw: RawPolicy = toml::from_str(text)
            .map_err(|error| PolicyErrors::one(PolicyError::new("policy", error.to_string())))?;
        Self::validate(raw, *context)
    }

    fn validate(raw: RawPolicy, context: PolicyContext) -> Result<Self, PolicyErrors> {
        let RawPolicy {
            schema,
            project,
            version,
            gates,
            build,
            publish,
            proof,
            workflows,
            targets,
        } = raw;
        let mut faults = Vec::new();

        if schema != SUPPORTED_SCHEMA {
            faults.push(PolicyError::new(
                "schema",
                format!(
                    "schema {schema} is not supported; this build implements {SUPPORTED_SCHEMA}"
                ),
            ));
        }

        let version_source = parse_version_source(&project.version_source, context, &mut faults);

        if !project.tag_format.contains(TAG_PLACEHOLDER) {
            faults.push(PolicyError::new(
                "project.tag-format",
                format!(
                    "`{}` does not carry {TAG_PLACEHOLDER}, so the tag cannot be the evidence \
                     anchor (decision-4)",
                    project.tag_format
                ),
            ));
        }

        let version = version.unwrap_or_default();
        let manifests = duplicate_free("version.manifests", &version.manifests, &mut faults);
        if manifests.is_empty() {
            faults.push(PolicyError::new(
                "version.manifests",
                "at least one manifest must be listed; `prepare` stages only what it owns",
            ));
        }
        let lockfiles = duplicate_free("version.lockfiles", &version.lockfiles, &mut faults);
        if version.changelog.trim().is_empty() {
            faults.push(PolicyError::new(
                "version.changelog",
                "a changelog path is required; Convco writes it (decision-6)",
            ));
        }

        let gates_declared = gates.is_some();
        let gates = gates.map(|gates| gates.required).unwrap_or_default();
        if gates_declared && gates.is_empty() {
            faults.push(PolicyError::new(
                "gates.required",
                "an empty gate list would let `cut` proceed with no proof",
            ));
        }

        let build = build.map(|build| build_policy(build, &mut faults));
        let proof = proof.map(|proof| proof_policy(proof, &mut faults));
        let workflows = workflows.map(|workflows| workflows_policy(workflows, &mut faults));
        let targets = targets
            .into_iter()
            .map(|target| target_policy(target, &mut faults))
            .collect();

        if faults.is_empty() {
            Ok(Self {
                schema,
                version_source,
                tag_format: project.tag_format,
                bot_identity: project.bot_identity,
                manifests,
                lockfiles,
                changelog: version.changelog,
                gates,
                build,
                publish_environment: publish.and_then(|publish| publish.environment),
                proof,
                workflows,
                targets,
            })
        } else {
            Err(PolicyErrors(faults))
        }
    }

    /// Builds the evidence a plan must record, refusing combinations that cannot be reproduced.
    ///
    /// # Errors
    ///
    /// Refuses Convco mode without evidence, and manifest mode that carries Convco evidence.
    pub fn plan_evidence(
        &self,
        convco: Option<ConvcoEvidence>,
    ) -> Result<PlanEvidence, PolicyError> {
        match (self.version_source, convco) {
            (VersionSource::Convco, Some(convco)) => Ok(PlanEvidence {
                schema: self.schema,
                version_source: self.version_source,
                convco: Some(convco),
            }),
            (VersionSource::Convco, None) => Err(PolicyError::new(
                "project.version-source",
                "convco mode records the Convco version and configuration hashes as plan evidence, \
                 and none were supplied",
            )),
            (VersionSource::Manifest, Some(_)) => Err(PolicyError::new(
                "project.version-source",
                "manifest mode cannot carry Convco evidence (decision-6)",
            )),
            (VersionSource::Manifest, None) => Ok(PlanEvidence {
                schema: self.schema,
                version_source: self.version_source,
                convco: None,
            }),
        }
    }

    /// The schema this policy declares.
    #[must_use]
    pub fn schema(&self) -> u32 {
        self.schema
    }

    /// Where the version comes from.
    #[must_use]
    pub fn version_source(&self) -> VersionSource {
        self.version_source
    }

    /// The tag format, carrying [`TAG_PLACEHOLDER`].
    #[must_use]
    pub fn tag_format(&self) -> &str {
        &self.tag_format
    }

    /// The identity `prepare` commits as, when configured.
    #[must_use]
    pub fn bot_identity(&self) -> Option<&str> {
        self.bot_identity.as_deref()
    }

    /// Files whose version literal `prepare` owns.
    #[must_use]
    pub fn manifests(&self) -> &[String] {
        &self.manifests
    }

    /// Lockfiles that must move with the manifests.
    #[must_use]
    pub fn lockfiles(&self) -> &[String] {
        &self.lockfiles
    }

    /// The changelog path Convco writes.
    #[must_use]
    pub fn changelog(&self) -> &str {
        &self.changelog
    }

    /// Gates that must pass before `cut`.
    #[must_use]
    pub fn gates(&self) -> &[String] {
        &self.gates
    }

    /// The build policy, when configured.
    #[must_use]
    pub fn build(&self) -> Option<&BuildPolicy> {
        self.build.as_ref()
    }

    /// The deployment environment that gates publishing.
    #[must_use]
    pub fn publish_environment(&self) -> Option<&str> {
        self.publish_environment.as_deref()
    }

    /// The proof policy, when configured.
    #[must_use]
    pub fn proof(&self) -> Option<&ProofPolicy> {
        self.proof.as_ref()
    }

    /// The workflow render policy, when configured.
    #[must_use]
    pub fn workflows(&self) -> Option<&WorkflowsPolicy> {
        self.workflows.as_ref()
    }

    /// Publication targets, in document order.
    #[must_use]
    pub fn targets(&self) -> &[TargetPolicy] {
        &self.targets
    }
}

/// The Convco version and configuration digests that make a plan reproducible.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConvcoEvidence {
    version: String,
    config_hashes: BTreeMap<String, String>,
}

impl ConvcoEvidence {
    /// Records evidence, refusing an empty version or an empty hash set.
    ///
    /// # Errors
    ///
    /// Refuses evidence that cannot identify what produced the version.
    pub fn new(
        version: impl Into<String>,
        config_hashes: BTreeMap<String, String>,
    ) -> Result<Self, PolicyError> {
        let version = version.into();
        if version.trim().is_empty() {
            return Err(PolicyError::new(
                "project.version-source",
                "the Convco version is required as evidence",
            ));
        }
        if config_hashes.is_empty() {
            return Err(PolicyError::new(
                "project.version-source",
                "at least one Convco configuration hash is required as evidence",
            ));
        }
        Ok(Self {
            version,
            config_hashes,
        })
    }

    /// The Convco version that produced the version.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// File name to SHA-256 digest of the Convco configuration.
    #[must_use]
    pub fn config_hashes(&self) -> &BTreeMap<String, String> {
        &self.config_hashes
    }
}

/// What a plan records about how the version was derived.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanEvidence {
    schema: u32,
    version_source: VersionSource,
    convco: Option<ConvcoEvidence>,
}

impl PlanEvidence {
    /// The policy schema the plan was derived from.
    #[must_use]
    pub fn schema(&self) -> u32 {
        self.schema
    }

    /// Where the version came from.
    #[must_use]
    pub fn version_source(&self) -> VersionSource {
        self.version_source
    }

    /// Convco evidence, present exactly when the source is Convco.
    #[must_use]
    pub fn convco(&self) -> Option<&ConvcoEvidence> {
        self.convco.as_ref()
    }
}

fn parse_version_source(
    value: &str,
    context: PolicyContext,
    faults: &mut Vec<PolicyError>,
) -> VersionSource {
    match value {
        "convco" => VersionSource::Convco,
        "manifest" if context.convco_configured => {
            faults.push(PolicyError::new(
                "project.version-source",
                "`manifest` is incompatible with an active Convco policy; Convco is the only \
                 version authority (decision-6)",
            ));
            VersionSource::Manifest
        }
        "manifest" => VersionSource::Manifest,
        other => {
            faults.push(PolicyError::new(
                "project.version-source",
                format!("`{other}` is not an accepted source; expected `convco` or `manifest`"),
            ));
            VersionSource::Manifest
        }
    }
}

fn duplicate_free(key: &str, values: &[String], faults: &mut Vec<PolicyError>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut kept = Vec::with_capacity(values.len());
    for value in values {
        if value.trim().is_empty() {
            faults.push(PolicyError::new(key, "an empty path is not a path"));
            continue;
        }
        if seen.insert(value.clone()) {
            kept.push(value.clone());
        } else {
            faults.push(PolicyError::new(
                key,
                format!("`{value}` is listed twice, which makes ownership ambiguous"),
            ));
        }
    }
    kept
}

fn build_policy(build: RawBuild, faults: &mut Vec<PolicyError>) -> BuildPolicy {
    if build.task.trim().is_empty() {
        faults.push(PolicyError::new(
            "build.task",
            "a build task is required; the controller runs the repository's task (decision-2)",
        ));
    }
    if build.environment.trim().is_empty() {
        faults.push(PolicyError::new(
            "build.environment",
            "a Pixi environment is required for the build task",
        ));
    }
    let matrix = build
        .matrix
        .into_iter()
        .enumerate()
        .map(|(index, row)| {
            if row.platform.trim().is_empty() {
                faults.push(PolicyError::new(
                    "build.matrix",
                    format!("entry {index} has no platform"),
                ));
            }
            if row.runner.trim().is_empty() {
                faults.push(PolicyError::new(
                    "build.matrix",
                    format!("entry {index} has no runner"),
                ));
            }
            if row.artifacts.is_empty() {
                faults.push(PolicyError::new(
                    "build.matrix",
                    format!("entry {index} declares no artifacts, so nothing can be collected"),
                ));
            }
            BuildRow {
                platform: row.platform,
                runner: row.runner,
                args: row.args,
                artifacts: row.artifacts,
            }
        })
        .collect();

    BuildPolicy {
        task: build.task,
        environment: build.environment,
        matrix,
    }
}

fn proof_policy(proof: RawProof, faults: &mut Vec<PolicyError>) -> ProofPolicy {
    if proof.enabled && proof.smoke.is_empty() {
        faults.push(PolicyError::new(
            "proof.smoke",
            "an enabled proof needs at least one smoke command to have anything to prove",
        ));
    }
    ProofPolicy {
        enabled: proof.enabled,
        isolated: proof.isolated,
        smoke: proof.smoke,
        expect_version: proof.expect_version,
    }
}

fn workflows_policy(workflows: RawWorkflows, faults: &mut Vec<PolicyError>) -> WorkflowsPolicy {
    for name in &workflows.render {
        if !RENDERABLE.contains(&name.as_str()) {
            faults.push(PolicyError::new(
                "workflows.render",
                format!(
                    "`{name}` is not rendered; the renderer claims the release lane only, so \
                     `ci.yml` and `docs.yml` stay hand-written (decision-10 §2)"
                ),
            ));
        }
    }
    WorkflowsPolicy {
        render: workflows.render,
        runner: workflows.runner,
    }
}

fn target_policy(target: RawTarget, faults: &mut Vec<PolicyError>) -> TargetPolicy {
    if target.name.trim().is_empty() {
        faults.push(PolicyError::new("target.name", "every target needs a name"));
    } else if target.name != ADMISSIBLE_TARGET {
        faults.push(PolicyError::new(
            "target.name",
            format!(
                "`{}` is not admissible; only `{ADMISSIBLE_TARGET}` is admitted before REL-6 \
                 (decision-9)",
                target.name
            ),
        ));
    }
    TargetPolicy {
        name: target.name,
        required: target.required,
        channel: target.channel,
        auth: target.auth,
        platforms: target.platforms,
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPolicy {
    schema: u32,
    project: RawProject,
    #[serde(default)]
    version: Option<RawVersion>,
    #[serde(default)]
    gates: Option<RawGates>,
    #[serde(default)]
    build: Option<RawBuild>,
    #[serde(default)]
    publish: Option<RawPublish>,
    #[serde(default)]
    proof: Option<RawProof>,
    #[serde(default)]
    workflows: Option<RawWorkflows>,
    #[serde(default, rename = "target")]
    targets: Vec<RawTarget>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct RawProject {
    version_source: String,
    tag_format: String,
    #[serde(default)]
    bot_identity: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawVersion {
    #[serde(default)]
    manifests: Vec<String>,
    #[serde(default)]
    lockfiles: Vec<String>,
    #[serde(default)]
    changelog: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGates {
    #[serde(default)]
    required: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBuild {
    task: String,
    environment: String,
    #[serde(default)]
    matrix: Vec<RawBuildRow>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBuildRow {
    platform: String,
    runner: String,
    #[serde(default)]
    args: Option<String>,
    #[serde(default)]
    artifacts: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPublish {
    #[serde(default)]
    environment: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct RawProof {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    isolated: bool,
    #[serde(default)]
    smoke: Vec<String>,
    #[serde(default)]
    expect_version: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWorkflows {
    #[serde(default)]
    render: Vec<String>,
    #[serde(default)]
    runner: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTarget {
    name: String,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    auth: Option<String>,
    #[serde(default)]
    platforms: Vec<String>,
}
