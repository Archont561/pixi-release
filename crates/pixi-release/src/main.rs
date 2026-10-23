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
use pixi_release_core::prepare::version_edits;
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
    /// Report the edits a release needs, without writing them
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
    /// Report the edits and write nothing; the only implemented mode
    #[arg(long, required = true)]
    dry_run: bool,
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
        Command::Prepare(options) => {
            run_prepare(&options.policy_path, &options.plan, options.out.as_deref())
        }
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

/// `prepare --dry-run` names the changes a prepare would make and makes none of them.
///
/// REL-4 ships the dry run only: writing the literals and regenerating the changelog land with
/// REL-8, so this command has no mode that touches a tracked file.
fn run_prepare(policy_path: &str, plan_path: &str, out: Option<&str>) -> Result<(), String> {
    let policy = policy::load(policy_path, policy::context())?;
    let plan = read_plan(plan_path)?;

    let mut manifests = Vec::new();
    for path in policy.manifests() {
        let text = fs::read_to_string(path).map_err(|error| format!("prepare: {path}: {error}"))?;
        manifests.push((path.clone(), text));
    }
    let edits = version_edits(manifests, plan.version())
        .map_err(|refusal| format!("prepare: {refusal}"))?;

    let mut touched: Vec<String> = edits.iter().map(|edit| edit.path().to_owned()).collect();
    touched.extend(policy.lockfiles().iter().cloned());
    touched.push(policy.changelog().to_owned());

    for edit in &edits {
        println!(
            "prepare: would update {}: {} -> {}",
            edit.path(),
            edit.from(),
            edit.to()
        );
    }
    println!(
        "prepare: {} manifest(s) would change, {} file(s) in the touched set, worktree untouched",
        edits.len(),
        {
            let mut unique = touched.clone();
            unique.sort();
            unique.dedup();
            unique.len()
        }
    );

    if let Some(path) = out {
        let recorded = plan.with_touched(touched).to_json();
        fs::write(path, recorded).map_err(|error| format!("prepare: {path}: {error}"))?;
        println!("prepare: recorded the touched set in {path}");
    }
    Ok(())
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
