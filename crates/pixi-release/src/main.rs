//! Thin command-line scaffold for the future `pixi release` extension.

mod git;
mod policy;

use std::fs;

use pixi_release_core::config::{ReleasePolicy, VersionSource};
use pixi_release_core::plan::{Bump, Plan, PlanInputs, SourceRevision, Version, VersionAuthority};
use pixi_release_git::{FakeGit, GitProtocol};

const USAGE: &str = "pixi-release <plan [--policy PATH] [--version V] [--bump auto|major|minor|patch] [--out PATH]|prepare|cut|verify|publish|resume|workflow check>";

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let exit_code = run(&arguments);
    std::process::exit(exit_code);
}

fn run(arguments: &[String]) -> i32 {
    match arguments {
        [command] if command == "--version" || command == "-V" => {
            println!("pixi-release {}", env!("CARGO_PKG_VERSION"));
            0
        }
        [command, rest @ ..] if command == "plan" => plan(rest),
        [command]
            if matches!(
                command.as_str(),
                "prepare" | "cut" | "verify" | "publish" | "resume"
            ) =>
        {
            println!("scaffold {command}: release mutation is intentionally not implemented");
            0
        }
        [workflow, check] if workflow == "workflow" && check == "check" => {
            let git = FakeGit::new("scaffold", true);
            println!("scaffold workflow check: git_clean={}", git.is_clean());
            0
        }
        _ => {
            eprintln!("{USAGE}");
            2
        }
    }
}

/// Everything `plan` accepts on the command line.
struct PlanOptions {
    policy_path: String,
    convco_version: Option<String>,
    explicit_version: Option<String>,
    bump: Bump,
    out: Option<String>,
    commit: Option<String>,
}

impl PlanOptions {
    /// Parses the flags, returning the usage message to show on a malformed invocation.
    fn parse(arguments: &[String]) -> Result<Self, String> {
        let mut options = Self {
            policy_path: ".pixi-release.toml".to_owned(),
            convco_version: None,
            explicit_version: None,
            bump: Bump::Auto,
            out: None,
            commit: None,
        };
        let mut rest = arguments.iter();
        while let Some(argument) = rest.next() {
            let value = |rest: &mut std::slice::Iter<'_, String>| {
                rest.next()
                    .cloned()
                    .ok_or_else(|| format!("plan: `{argument}` needs a value"))
            };
            match argument.as_str() {
                "--policy" => options.policy_path = value(&mut rest)?,
                "--convco-version" => options.convco_version = Some(value(&mut rest)?),
                "--version" => options.explicit_version = Some(value(&mut rest)?),
                "--out" => options.out = Some(value(&mut rest)?),
                "--commit" => options.commit = Some(value(&mut rest)?),
                "--bump" => {
                    let requested = value(&mut rest)?;
                    options.bump = Bump::parse(&requested).ok_or_else(|| {
                        format!("plan: `{requested}` is not one of auto, major, minor, patch")
                    })?;
                }
                other => return Err(format!("plan: unknown argument `{other}`")),
            }
        }
        Ok(options)
    }
}

/// `plan` derives the immutable release proposal and records where every part of it came from.
///
/// It reads the policy and two Git facts and writes nothing but the requested `--out` file: no
/// commit, no tag, no workflow (`doc-2` §2.3, REL-3).
fn plan(arguments: &[String]) -> i32 {
    let options = match PlanOptions::parse(arguments) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}\n{USAGE}");
            return 2;
        }
    };
    match run_plan(&options) {
        Ok(()) => 0,
        Err(message) => {
            eprintln!("{message}");
            1
        }
    }
}

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
    let derived = policy::convco_next_version(options.bump)?;
    let version = Version::parse(&derived).map_err(|error| format!("plan: {error}"))?;
    let evidence = policy::convco_evidence(options.convco_version.as_deref())?;
    Ok(VersionAuthority::Convco {
        bump: options.bump,
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
