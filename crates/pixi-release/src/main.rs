//! Thin command-line scaffold for the future `pixi release` extension.

use pixi_release_core::{ReleasePhase, ReleasePlan};
use pixi_release_git::{FakeGit, GitProtocol};

const USAGE: &str = "pixi-release <plan|prepare|cut|verify|publish|resume|workflow check>";

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
        [command] if command == "plan" => {
            let plan = ReleasePlan::new("0.1.0", vec!["github-release".into(), "prefix".into()]);
            println!(
                "scaffold {}: version={} targets={:?}",
                phase_name(ReleasePhase::Plan),
                plan.version(),
                plan.targets()
            );
            0
        }
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

const fn phase_name(phase: ReleasePhase) -> &'static str {
    match phase {
        ReleasePhase::Plan => "plan",
        ReleasePhase::Prepare => "prepare",
        ReleasePhase::Cut => "cut",
        ReleasePhase::Verify => "verify",
        ReleasePhase::Publish => "publish",
        ReleasePhase::Resume => "resume",
    }
}
