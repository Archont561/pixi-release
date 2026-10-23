//! Thin command-line scaffold for the future `pixi release` extension.

mod policy;

use pixi_release_core::config::VersionSource;
use pixi_release_git::{FakeGit, GitProtocol};

const USAGE: &str = "pixi-release <plan [--policy PATH] [--convco-version V]|prepare|cut|verify|\
publish|resume|workflow check>";

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

/// `plan` validates the authored policy and records how the version was derived. It performs no
/// release mutation: this is the phase that only inspects (`doc-1`).
fn plan(arguments: &[String]) -> i32 {
    let mut policy_path = ".pixi-release.toml".to_owned();
    let mut convco_version: Option<String> = None;
    let mut rest = arguments.iter();
    while let Some(argument) = rest.next() {
        match argument.as_str() {
            "--policy" => {
                let Some(path) = rest.next() else {
                    eprintln!("plan: --policy needs a path");
                    return 2;
                };
                policy_path.clone_from(path);
            }
            "--convco-version" => {
                let Some(version) = rest.next() else {
                    eprintln!("plan: --convco-version needs a value");
                    return 2;
                };
                convco_version = Some(version.clone());
            }
            other => {
                eprintln!("plan: unknown argument `{other}`\n{USAGE}");
                return 2;
            }
        }
    }

    let context = policy::context();
    let policy = match policy::load(&policy_path, context) {
        Ok(policy) => policy,
        Err(rendered) => {
            eprintln!("{rendered}");
            return 1;
        }
    };

    let evidence = if policy.version_source() == VersionSource::Convco {
        match policy::convco_evidence(convco_version.as_deref()) {
            Ok(evidence) => Some(evidence),
            Err(message) => {
                eprintln!("{message}");
                return 1;
            }
        }
    } else {
        None
    };

    match policy.plan_evidence(evidence) {
        Ok(plan) => {
            println!("{}", policy::render_evidence(&plan, &policy));
            0
        }
        Err(error) => {
            eprintln!("plan: {error}");
            1
        }
    }
}
