//! Command-line entry point for the `pixi release` extension.
//!
//! `clap` owns the grammar: which subcommands exist, which flags they accept, and what a
//! malformed invocation prints. Nothing else in the crate parses `argv`, so the usage text and
//! the exit codes come from one place instead of a hand-written matcher.

mod gates;
mod git;
mod policy;

use std::fs;

use clap::{Parser, Subcommand, ValueEnum};
use pixi_release_core::config::{ReleasePolicy, VersionSource};
use pixi_release_core::cut::{self, check_repository};
use pixi_release_core::plan::{Bump, Plan, PlanInputs, SourceRevision, Version, VersionAuthority};
use pixi_release_core::prepare::{
    Edit, Stage, changelog_target, pin_edits, stage, stages, version_edits,
};
use pixi_release_git::LocalGit;
use pixi_release_git::{FakeGit, GitProtocol};

#[derive(Parser)]
#[command(
    name = "pixi-release",
    version,
    about = "Safe release workflow controller for Pixi projects",
    subcommand_required = true,
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Derive the immutable release plan
    Plan(PlanOptions),
    /// Stage the planned literals and the Convco changelog (--dry-run reports only)
    Prepare(PrepareOptions),
    /// Validate the plan, then commit and tag it
    Cut(CutOptions),
    /// Verify recorded release evidence
    Verify,
    /// Publish verified artifacts
    Publish,
    /// Resume unfinished publish targets
    Resume,
    /// Generated workflow maintenance
    Workflow {
        #[command(subcommand)]
        command: WorkflowCommand,
    },
}

#[derive(Subcommand)]
enum WorkflowCommand {
    /// Check generated workflows for drift
    Check,
}

/// The bump Convco should apply, as a command-line value.
#[derive(Clone, Copy, ValueEnum)]
enum BumpChoice {
    Auto,
    Major,
    Minor,
    Patch,
}

impl From<BumpChoice> for Bump {
    fn from(choice: BumpChoice) -> Self {
        match choice {
            BumpChoice::Auto => Self::Auto,
            BumpChoice::Major => Self::Major,
            BumpChoice::Minor => Self::Minor,
            BumpChoice::Patch => Self::Patch,
        }
    }
}

/// Everything `plan` accepts on the command line.
#[derive(Parser)]
struct PlanOptions {
    /// Release policy to read
    #[arg(long, value_name = "PATH", default_value = ".pixi-release.toml")]
    policy_path: String,
    /// Convco version to record as plan evidence
    #[arg(long, value_name = "V")]
    convco_version: Option<String>,
    /// Version to use instead of letting Convco derive one
    #[arg(long = "version", value_name = "V")]
    explicit_version: Option<String>,
    /// Version bump Convco applies
    #[arg(long, value_enum, default_value_t = BumpChoice::Auto)]
    bump: BumpChoice,
    /// Write the plan here instead of stdout
    #[arg(long, value_name = "PATH")]
    out: Option<String>,
    /// Commit the plan describes, instead of HEAD
    #[arg(long, value_name = "SHA", hide = true)]
    commit: Option<String>,
}

/// Everything `prepare` accepts on the command line.
#[derive(Parser)]
struct PrepareOptions {
    /// The recorded plan to prepare for
    #[arg(long, value_name = "PATH")]
    plan: String,
    /// Release policy to read
    #[arg(long, value_name = "PATH", default_value = ".pixi-release.toml")]
    policy_path: String,
    /// Report the work and write nothing
    #[arg(long)]
    dry_run: bool,
    /// The changelog to stage; must be the path the policy names
    #[arg(long, value_name = "PATH")]
    changelog: Option<String>,
    /// Record the touched set into this plan file
    #[arg(long, value_name = "PATH")]
    out: Option<String>,
}

/// Everything `cut` accepts on the command line.
#[derive(Parser)]
struct CutOptions {
    /// The recorded plan to cut
    #[arg(long, value_name = "PATH")]
    plan: String,
    /// Release policy to read
    #[arg(long, value_name = "PATH", default_value = ".pixi-release.toml")]
    policy_path: String,
    /// Validate and report the operations without executing them
    #[arg(long)]
    dry_run: bool,
    /// Push the release commit and tag after cutting them
    #[arg(long)]
    push: bool,
}

fn main() {
    std::process::exit(run(Cli::parse()));
}

fn run(cli: Cli) -> i32 {
    let outcome = match cli.command {
        Command::Plan(options) => run_plan(&options),
        Command::Prepare(options) => run_prepare(&options),
        Command::Cut(options) => run_cut(
            &options.policy_path,
            &options.plan,
            options.dry_run,
            options.push,
        ),
        Command::Verify => {
            scaffold("verify");
            Ok(())
        }
        Command::Publish => {
            scaffold("publish");
            Ok(())
        }
        Command::Resume => {
            scaffold("resume");
            Ok(())
        }
        Command::Workflow {
            command: WorkflowCommand::Check,
        } => {
            workflow_check();
            Ok(())
        }
    };
    match outcome {
        Ok(()) => 0,
        Err(message) => {
            eprintln!("{message}");
            1
        }
    }
}

/// A lifecycle phase that is named but not implemented yet.
fn scaffold(command: &str) {
    println!("scaffold {command}: release mutation is intentionally not implemented");
}

/// The drift gate's placeholder: it reports what it checked and claims nothing more.
fn workflow_check() {
    let git = FakeGit::new("scaffold", true);
    println!("scaffold workflow check: git_clean={}", git.is_clean());
}

/// `plan` derives the immutable release proposal and records where every part of it came from.
///
/// It reads the policy and two Git facts and writes nothing but the requested `--out` file: no
/// commit, no tag, no workflow (`doc-2` §2.3, REL-3).
fn run_plan(options: &PlanOptions) -> Result<(), String> {
    let policy = policy::load(&options.policy_path, policy::context())?;
    let policy_digest = policy::digest(&options.policy_path)?;
    let commit = match &options.commit {
        Some(commit) => commit.clone(),
        None => git::head_sha()?,
    };
    let source =
        SourceRevision::new(commit, git::is_clean()?).map_err(|error| format!("plan: {error}"))?;
    let inputs = PlanInputs {
        source,
        policy_digest,
        tool_version: env!("CARGO_PKG_VERSION").to_owned(),
    };
    let plan = Plan::propose(&policy, inputs, resolve_authority(&policy, options)?)
        .map_err(|error| format!("plan: {error}"))?;
    emit(&plan, options.out.as_deref())
}

/// Decides where the version comes from, without ever deriving one itself (`decision-6`).
fn resolve_authority(
    policy: &ReleasePolicy,
    options: &PlanOptions,
) -> Result<VersionAuthority, String> {
    if let Some(requested) = &options.explicit_version {
        let version = Version::parse(requested).map_err(|error| format!("plan: {error}"))?;
        return Ok(VersionAuthority::Explicit { version });
    }
    if policy.version_source() != VersionSource::Convco {
        return Err(
            "plan: project.version-source: a manifest policy needs an explicit --version"
                .to_owned(),
        );
    }
    let bump = Bump::from(options.bump);
    let derived = policy::convco_next_version(bump)?;
    let version = Version::parse(&derived).map_err(|error| format!("plan: {error}"))?;
    let evidence = policy::convco_evidence(options.convco_version.as_deref())?;
    Ok(VersionAuthority::Convco {
        bump,
        version,
        evidence,
    })
}

/// The number of distinct paths in a touched set, which may name one file twice.
fn unique_count(paths: &[String]) -> usize {
    let mut unique = paths.to_vec();
    unique.sort();
    unique.dedup();
    unique.len()
}

/// Writes the plan where the caller asked, or prints it, and does nothing else.
fn emit(plan: &Plan, out: Option<&str>) -> Result<(), String> {
    if let Some(path) = out {
        fs::write(path, plan.to_json()).map_err(|error| format!("plan: {path}: {error}"))?;
        println!(
            "plan: wrote {path} ({} at tag {})",
            plan.version(),
            plan.tag()
        );
    } else {
        print!("{}", plan.to_json());
    }
    Ok(())
}

/// `prepare` writes the plan into the worktree — the version literals it names, the pins that
/// quote them, the lockfiles Cargo regenerates, and the changelog Convco generates — and
/// `--dry-run` reports the same work without touching a tracked file (REL-8, REL-29).
///
/// Every refusal is raised before the first byte is written, so a refused prepare leaves the
/// worktree exactly as it was. The staged changelog is Convco's own bytes: this command writes no
/// release-note text of its own (`decision-6`).
fn run_prepare(options: &PrepareOptions) -> Result<(), String> {
    let policy = policy::load(&options.policy_path, policy::context())?;
    let plan = read_plan(&options.plan)?;

    let changelog = changelog_target(policy.changelog(), options.changelog.as_deref())
        .map_err(|refusal| format!("prepare: {refusal}"))?;

    let mut manifests = Vec::new();
    for path in policy.manifests() {
        let text = fs::read_to_string(path).map_err(|error| format!("prepare: {path}: {error}"))?;
        manifests.push((path.clone(), text));
    }
    let edits = version_edits(manifests.clone(), plan.version())
        .map_err(|refusal| format!("prepare: {refusal}"))?;
    // Computed against the text as read: the pin stage keeps line numbers, so it lands on the
    // already-stamped document without a second read.
    let pins = pin_edits(manifests.clone(), plan.version())
        .map_err(|refusal| format!("prepare: {refusal}"))?;
    let (lockfiles, unrefreshed) = lockfile_stages(&policy, &edits[..], &pins[..]);
    let body = policy::convco_changelog(plan.version().as_str())?;

    let writes = Writes {
        manifests,
        edits,
        pins,
        lockfiles,
        unrefreshed,
        changelog,
        body,
        version: plan.version().as_str().to_owned(),
    };
    writes.report(options.dry_run);
    if !options.dry_run {
        writes.apply()?;
    }

    if let Some(path) = options.out.as_deref() {
        let recorded = plan.with_touched(writes.touched()).to_json();
        fs::write(path, recorded).map_err(|error| format!("prepare: {path}: {error}"))?;
        println!("prepare: recorded the touched set in {path}");
    }
    Ok(())
}

/// Everything one prepare writes, decided before the first byte is written.
///
/// The fields mirror the stage sequence: the manifest literals, the pins beside them, the lockfiles
/// that record the result, and the changelog. `manifests` keeps the text as read, because both
/// editing stages rewrite the same document.
struct Writes {
    manifests: Vec<(String, String)>,
    edits: Vec<Edit>,
    pins: Vec<Edit>,
    lockfiles: Vec<String>,
    unrefreshed: Vec<String>,
    changelog: String,
    /// Convco's own bytes, written unchanged (`decision-6`).
    body: Vec<u8>,
    version: String,
}

impl Writes {
    /// The paths this prepare claims to have written, in stage order.
    fn touched(&self) -> Vec<String> {
        let mut touched: Vec<String> = self
            .edits
            .iter()
            .map(|edit| edit.path().to_owned())
            .collect();
        touched.extend(self.pins.iter().map(|edit| edit.path().to_owned()));
        touched.extend(self.lockfiles.iter().cloned());
        touched.push(self.changelog.clone());
        touched
    }

    /// Prints one line per stage, in the order the core fixes ([`stages`]) rather than the order the
    /// edits happen to be listed: the output is the only place an operator sees which step writes
    /// what, so it has to be the order the steps run in.
    fn report(&self, dry_run: bool) {
        let verb = if dry_run { "would update" } else { "updated" };
        for step in stages() {
            match step {
                Stage::Manifests => {
                    for edit in &self.edits {
                        println!(
                            "prepare: {verb} {}: {} -> {}",
                            edit.path(),
                            edit.from(),
                            edit.to()
                        );
                    }
                }
                Stage::Pins => {
                    for edit in &self.pins {
                        println!(
                            "prepare: {verb} {}:{}: pin {} -> {}",
                            edit.path(),
                            edit.line().map_or(0, |line| line + 1),
                            edit.from(),
                            edit.to()
                        );
                    }
                }
                Stage::Generated => {}
                Stage::Lockfiles => self.report_lockfiles(dry_run),
                Stage::Changelog => {
                    println!(
                        "prepare: {} {} ({} bytes from convco {})",
                        if dry_run { "would stage" } else { "staged" },
                        self.changelog,
                        self.body.len(),
                        self.version
                    );
                }
            }
        }
        if dry_run {
            println!(
                "prepare: {} manifest(s) and {} pin(s) would change, {} file(s) in the touched set, \
                 worktree untouched",
                self.edits.len(),
                self.pins.len(),
                unique_count(&self.touched())
            );
        }
    }

    /// The lockfile stage: what will be refreshed by Cargo, and what this release cannot refresh.
    fn report_lockfiles(&self, dry_run: bool) {
        for path in &self.lockfiles {
            println!(
                "prepare: {} {path} (cargo update --workspace, after the manifests agree)",
                if dry_run {
                    "would refresh"
                } else {
                    "refreshed"
                }
            );
        }
        for path in &self.unrefreshed {
            println!(
                "prepare: not refreshing {path}: no refresher for this format yet, so the plan \
                 does not record it (REL-29)"
            );
        }
    }

    /// Applies the stages in order: manifests, then the pins in the same documents, then the
    /// lockfiles that record them, then the changelog.
    fn apply(&self) -> Result<(), String> {
        let mut staged: std::collections::BTreeMap<String, String> =
            self.manifests.iter().cloned().collect();
        // Staging preserves the line count, so a pin's recorded line still points at its own
        // literal after the manifest stage has run.
        for edit in self.edits.iter().chain(self.pins.iter()) {
            let text = staged
                .get_mut(edit.path())
                .ok_or_else(|| format!("prepare: {} was listed but not read", edit.path()))?;
            *text = stage(text, edit);
        }
        for (path, text) in &staged {
            fs::write(path, text).map_err(|error| format!("prepare: {path}: {error}"))?;
        }
        for path in &self.lockfiles {
            refresh_lockfile(path)?;
            println!("prepare: refreshed {path} (cargo update --workspace)");
        }
        fs::write(&self.changelog, &self.body)
            .map_err(|error| format!("prepare: {}: {error}", self.changelog))?;
        println!(
            "prepare: staged {} manifest(s), {} pin(s) and {}",
            self.edits.len(),
            self.pins.len(),
            self.changelog
        );
        Ok(())
    }
}

/// Splits the declared lockfiles into the ones Cargo can refresh and the ones this release can only
/// name: a silently stale lock is the failure the stage exists to prevent, so an unsupported format
/// is printed rather than skipped (`REL-29`).
fn lockfile_stages(
    policy: &ReleasePolicy,
    edits: &[Edit],
    pins: &[Edit],
) -> (Vec<String>, Vec<String>) {
    let cargo_changed = edits
        .iter()
        .chain(pins.iter())
        .any(|edit| edit.path().ends_with("Cargo.toml"));
    let mut refreshable = Vec::new();
    let mut unrefreshed = Vec::new();
    for path in policy.lockfiles() {
        if path.as_str() == "Cargo.lock" {
            if cargo_changed {
                refreshable.push(path.clone());
            }
        } else {
            unrefreshed.push(path.clone());
        }
    }
    (refreshable, unrefreshed)
}

/// Refreshes a Cargo lockfile so it records the versions the manifests now carry.
///
/// `--workspace` moves the members and nothing else, so no dependency is upgraded by a release.
/// Offline is tried first because the vendored registry is enough when no new dependency appears;
/// the online retry covers a machine that has one.
fn refresh_lockfile(path: &str) -> Result<(), String> {
    if path != "Cargo.lock" {
        return Err(format!(
            "prepare: {path}: no refresher knows this lockfile format yet (REL-29)"
        ));
    }
    let mut failure = String::from("cargo update never ran");
    for offline in [true, false] {
        let mut command = std::process::Command::new("cargo");
        command.args(["update", "--workspace"]);
        if offline {
            command.arg("--offline");
        }
        match command
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
        {
            Ok(status) if status.success() => return Ok(()),
            Ok(status) => failure = format!("cargo update exited {status}"),
            Err(error) => failure = format!("cargo: {error}"),
        }
    }
    Err(format!(
        "prepare: {path}: could not refresh the lockfile ({failure})"
    ))
}

/// `cut` validates everything, then stages exactly the plan's touched set, commits and tags.
fn run_cut(policy_path: &str, plan_path: &str, dry_run: bool, push: bool) -> Result<(), String> {
    let policy = policy::load(policy_path, policy::context())?;
    let policy_digest = policy::digest(policy_path)?;
    let plan = read_plan(plan_path)?;
    let mut repository = LocalGit::open(".").map_err(|error| format!("cut: {error}"))?;

    // Refuse what the repository alone can answer before spending the configured gates.
    check_repository(&repository, &policy, &plan, &policy_digest)
        .map_err(|refusal| format!("cut: {refusal}"))?;

    let gates = gates::report(&gates::PixiRunner, policy.gates());
    let permit = cut::permit(&repository, &policy, &plan, &policy_digest, &gates)
        .map_err(|refusal| format!("cut: {refusal}"))?;

    if dry_run {
        println!(
            "cut: would stage {} path(s), commit, and tag {} at {} (dry run, nothing written)",
            plan.touched().len(),
            permit.tag(),
            permit.commit()
        );
        return Ok(());
    }

    let outcome =
        cut::cut(&mut repository, &plan, &permit, push).map_err(|error| format!("cut: {error}"))?;
    println!(
        "cut: committed {} and tagged {}{}",
        outcome.commit(),
        outcome.tag(),
        if outcome.pushed() { " (pushed)" } else { "" }
    );
    Ok(())
}

/// Reads a plan file and re-validates it, so a hand-edited plan is refused rather than trusted.
fn read_plan(path: &str) -> Result<Plan, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("plan: {path}: {error}"))?;
    Plan::from_json(&text).map_err(|error| format!("plan: {path}: {error}"))
}
