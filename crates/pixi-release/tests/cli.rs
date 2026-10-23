use std::path::Path;
use std::process::Command;

use rstest::{fixture, rstest};

#[fixture]
fn cli_bin() -> &'static str {
    env!("CARGO_BIN_EXE_pixi-release")
}

/// The documented policy shape, written into a scratch repository root.
const POLICY: &str = r#"
schema = 1

[project]
version-source = "convco"
tag-format     = "v{version}"
bot-identity   = "pixi-release[bot]"

[version]
manifests = ["Cargo.toml"]
lockfiles = ["Cargo.lock"]
changelog = "CHANGELOG.md"

[gates]
required = ["gates"]

[workflows]
render = ["auto-release", "release"]
runner = "ubuntu-latest"

[[target]]
name     = "github-release"
required = true
"#;

/// A policy with `from` replaced by `to`, for the refusal cases.
fn policy_with(from: &str, to: &str) -> String {
    POLICY.replace(from, to)
}

/// Runs `git` in `dir`, returning stdout, and panicking on failure.
fn git(dir: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(dir)
        .output()
        .expect("git should run");
    assert!(
        output.status.success(),
        "git {arguments:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

/// Creates a single-commit repository in `dir` and returns the commit id.
fn init_repo(dir: &Path) -> String {
    git(dir, &["init", "--quiet"]);
    git(dir, &["config", "user.email", "test@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    git(dir, &["add", "--all"]);
    git(dir, &["commit", "--quiet", "--message", "chore: init"]);
    git(dir, &["rev-parse", "HEAD"])
}

/// Writes a stand-in `convco` that reports a fixed current version and a fixed next version.
fn fake_convco(dir: &Path, next: &str) -> std::ffi::OsString {
    let bin = dir.join("bin");
    std::fs::create_dir_all(&bin).expect("bin directory");
    let script = format!(
        "#!/bin/sh\ncase \"$1\" in\n  --version) echo \"convco 0.7.2\" ;;\n  version) echo \"{next}\" ;;\n  *) echo \"unexpected: $*\" >&2; exit 1 ;;\nesac\n"
    );
    let path = bin.join("convco");
    std::fs::write(&path, script).expect("fake convco");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&path, permissions).expect("executable bit");
    }
    let mut search = bin.into_os_string();
    if let Some(inherited) = std::env::var_os("PATH") {
        search.push(":");
        search.push(inherited);
    }
    search
}

#[rstest]
fn version_is_a_stable_cli_surface(cli_bin: &str) {
    let output = Command::new(cli_bin)
        .arg("--version")
        .output()
        .expect("scaffold binary should run");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "pixi-release 0.1.0\n"
    );
}

#[rstest]
fn unknown_command_is_refused(cli_bin: &str) {
    let output = Command::new(cli_bin)
        .arg("unknown")
        .output()
        .expect("scaffold binary should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("pixi-release <"));
}

/// AC#2: an explicit plan writes JSON that names the authority, the revision and the policy digest.
#[rstest]
fn plan_writes_a_plan_that_names_its_authority_and_source(cli_bin: &str) {
    let dir = tempfile::tempdir().expect("temp dir");
    let out = dir.path().join("plan.json");
    std::fs::write(dir.path().join(".pixi-release.toml"), POLICY).expect("policy");
    let head = init_repo(dir.path());

    let output = Command::new(cli_bin)
        .args([
            "plan",
            "--version",
            "0.2.0",
            "--out",
            out.to_str().expect("utf-8 path"),
        ])
        .current_dir(dir.path())
        .output()
        .expect("binary should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan = std::fs::read_to_string(&out).expect("plan file");
    assert!(plan.ends_with('\n'), "the plan ends in one newline");
    assert!(plan.contains("\"version\": \"0.2.0\""), "{plan}");
    assert!(plan.contains("\"tag\": \"v0.2.0\""), "{plan}");
    assert!(plan.contains("\"kind\": \"explicit\""), "{plan}");
    assert!(plan.contains(&head), "the commit id is recorded: {plan}");
    assert!(plan.contains("\"digest\": \"sha256:"), "{plan}");
    assert!(plan.contains("\"name\": \"github-release\""), "{plan}");
    assert!(plan.contains("\"version\": \"0.1.0\""), "{plan}");
}

/// AC#2, Convco half: the version comes from the authority, and its evidence travels with it.
#[rstest]
fn plan_derives_the_version_from_convco(cli_bin: &str) {
    let dir = tempfile::tempdir().expect("temp dir");
    std::fs::write(dir.path().join(".pixi-release.toml"), POLICY).expect("policy");
    std::fs::write(dir.path().join(".versionrc"), "{}").expect("versionrc");
    init_repo(dir.path());
    let path = fake_convco(dir.path(), "0.3.0");

    let output = Command::new(cli_bin)
        .args(["plan", "--bump", "minor"])
        .current_dir(dir.path())
        .env("PATH", path)
        .output()
        .expect("binary should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan = String::from_utf8_lossy(&output.stdout);
    assert!(plan.contains("\"version\": \"0.3.0\""), "{plan}");
    assert!(plan.contains("\"tag\": \"v0.3.0\""), "{plan}");
    assert!(plan.contains("\"bump\": \"minor\""), "{plan}");
    assert!(plan.contains("\"kind\": \"convco\""), "{plan}");
    assert!(plan.contains("\"version\": \"0.7.2\""), "{plan}");
    assert!(plan.contains("\".versionrc\""), "{plan}");
}

/// AC#3: `plan` mutates nothing — same `HEAD`, no tag, and only the file it was asked to write.
#[rstest]
fn plan_leaves_the_repository_untouched(cli_bin: &str) {
    let dir = tempfile::tempdir().expect("temp dir");
    let scratch = tempfile::tempdir().expect("scratch dir");
    let out = scratch.path().join("plan.json");
    std::fs::write(dir.path().join(".pixi-release.toml"), POLICY).expect("policy");
    let head = init_repo(dir.path());
    let before = git(dir.path(), &["status", "--porcelain"]);

    let output = Command::new(cli_bin)
        .args([
            "plan",
            "--version",
            "0.2.0",
            "--out",
            out.to_str().expect("utf-8 path"),
        ])
        .current_dir(dir.path())
        .output()
        .expect("binary should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(git(dir.path(), &["rev-parse", "HEAD"]), head, "HEAD moved");
    assert_eq!(git(dir.path(), &["tag", "--list"]), "", "a tag appeared");
    assert_eq!(
        git(dir.path(), &["status", "--porcelain"]),
        before,
        "the worktree changed"
    );
    assert!(
        !dir.path().join(".github").exists(),
        "no workflow was written"
    );
    assert!(out.exists(), "the requested file is the only output");
}

/// An unknown policy key is still refused through the binary, naming the key.
#[rstest]
fn plan_refuses_an_unknown_key(cli_bin: &str) {
    let dir = tempfile::tempdir().expect("temp dir");
    let policy = format!("{POLICY}\n[unexpected]\nvalue = 1\n");
    std::fs::write(dir.path().join(".pixi-release.toml"), policy).expect("policy");
    init_repo(dir.path());

    let output = Command::new(cli_bin)
        .args(["plan", "--version", "0.2.0"])
        .current_dir(dir.path())
        .output()
        .expect("binary should run");

    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("unexpected"),
        "the refusal names the key"
    );
}

/// A manifest policy cannot derive a version, so it must be told one.
#[rstest]
fn plan_refuses_a_manifest_policy_without_an_explicit_version(cli_bin: &str) {
    let dir = tempfile::tempdir().expect("temp dir");
    let policy = policy_with(
        "version-source = \"convco\"",
        "version-source = \"manifest\"",
    );
    std::fs::write(dir.path().join(".pixi-release.toml"), policy).expect("policy");
    init_repo(dir.path());

    let output = Command::new(cli_bin)
        .arg("plan")
        .current_dir(dir.path())
        .output()
        .expect("binary should run");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("project.version-source"), "{stderr}");
    assert!(stderr.contains("--version"), "{stderr}");
}

/// A missing policy is reported, not defaulted.
#[rstest]
fn plan_reports_a_missing_policy(cli_bin: &str) {
    let dir = tempfile::tempdir().expect("temp dir");
    let output = Command::new(cli_bin)
        .arg("plan")
        .current_dir(dir.path())
        .output()
        .expect("binary should run");

    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(".pixi-release.toml"),
        "the missing path must be named"
    );
}
