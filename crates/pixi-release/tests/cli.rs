use std::path::{Path, PathBuf};
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

/// The grammar is `clap`'s: `--help` lists every lifecycle command, and it exits successfully.
#[rstest]
fn help_lists_the_lifecycle_commands(cli_bin: &str) {
    let output = Command::new(cli_bin)
        .arg("--help")
        .output()
        .expect("binary should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "plan", "prepare", "cut", "verify", "publish", "resume", "workflow",
    ] {
        assert!(
            stdout.contains(command),
            "`{command}` missing from:\n{stdout}"
        );
    }
    let plan_help = Command::new(cli_bin)
        .args(["plan", "--help"])
        .output()
        .expect("binary should run");
    assert!(plan_help.status.success());
    let plan_help = String::from_utf8_lossy(&plan_help.stdout);
    assert!(plan_help.contains("--policy"), "{plan_help}");
    assert!(plan_help.contains("--bump"), "{plan_help}");
}

/// A flag a subcommand does not define is a usage error, not a silent no-op.
#[rstest]
fn an_unknown_flag_is_refused_with_usage(cli_bin: &str) {
    let output = Command::new(cli_bin)
        .args(["cut", "--plan", "plan.json", "--publish"])
        .output()
        .expect("binary should run");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--publish"), "{stderr}");
    assert!(stderr.contains("Usage: pixi-release cut"), "{stderr}");
}

/// The bump vocabulary is closed, and clap rejects anything outside it before the command runs.
#[rstest]
fn an_unknown_bump_is_refused(cli_bin: &str) {
    let dir = tempfile::tempdir().expect("temp dir");
    std::fs::write(dir.path().join(".pixi-release.toml"), POLICY).expect("policy");
    init_repo(dir.path());

    let output = Command::new(cli_bin)
        .args(["plan", "--bump", "sideways"])
        .current_dir(dir.path())
        .output()
        .expect("binary should run");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("sideways"), "{stderr}");
    assert!(
        stderr.contains("auto"),
        "the accepted set is listed: {stderr}"
    );
}

/// `cut` cannot run without a plan, and says so before touching the repository.
#[rstest]
fn cut_without_a_plan_is_a_usage_error(cli_bin: &str) {
    let output = Command::new(cli_bin)
        .arg("cut")
        .output()
        .expect("binary should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--plan"));
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

/// A policy with no gate list, so the CLI tests exercise the release logic rather than `pixi`.
const UNGATED_POLICY: &str = r#"
schema = 1

[project]
version-source = "convco"
tag-format     = "v{version}"

[version]
manifests = ["Cargo.toml"]
lockfiles = ["Cargo.lock"]
changelog = "CHANGELOG.md"

[[target]]
name     = "github-release"
required = true
"#;

/// Prepares a repository carrying the ungated policy and the three files the plan touches.
fn release_repo() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("temp dir");
    let scratch = tempfile::tempdir().expect("scratch dir");
    std::fs::write(dir.path().join(".pixi-release.toml"), UNGATED_POLICY).expect("policy");
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest");
    std::fs::write(dir.path().join("Cargo.lock"), "# lock\n").expect("lockfile");
    std::fs::write(dir.path().join("CHANGELOG.md"), "# Changelog\n").expect("changelog");
    init_repo(dir.path());
    (scratch, dir.keep())
}

/// Writes a plan for the prepared repository, returning the path.
fn write_plan(cli_bin: &str, repo: &Path, out: &Path) -> String {
    let output = Command::new(cli_bin)
        .args([
            "plan",
            "--version",
            "0.2.0",
            "--out",
            out.to_str().expect("utf-8 path"),
        ])
        .current_dir(repo)
        .output()
        .expect("binary should run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::read_to_string(out).expect("plan file")
}

/// AC#1: `prepare --dry-run` reports the intended change and leaves the worktree untouched.
#[rstest]
fn prepare_dry_run_reports_edits_without_touching_the_worktree(cli_bin: &str) {
    let (scratch, repo) = release_repo();
    let plan = scratch.path().join("plan.json");
    let recorded = scratch.path().join("prepared.json");
    write_plan(cli_bin, &repo, &plan);
    let before = std::fs::read_to_string(repo.join("Cargo.toml")).expect("manifest");
    let status_before = git(&repo, &["status", "--porcelain"]);

    let output = Command::new(cli_bin)
        .args([
            "prepare",
            "--plan",
            plan.to_str().expect("utf-8 path"),
            "--dry-run",
            "--out",
            recorded.to_str().expect("utf-8 path"),
        ])
        .current_dir(&repo)
        .output()
        .expect("binary should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("would update Cargo.toml: 0.1.0 -> 0.2.0"),
        "{stdout}"
    );
    assert!(stdout.contains("worktree untouched"), "{stdout}");
    assert!(
        stdout.contains("would stage CHANGELOG.md"),
        "the dry run must name the changelog it would write: {stdout}"
    );
    assert_eq!(
        std::fs::read_to_string(repo.join("Cargo.toml")).expect("manifest"),
        before,
        "the manifest was not rewritten"
    );
    assert_eq!(git(&repo, &["status", "--porcelain"]), status_before);
    let recorded_text = std::fs::read_to_string(&recorded).expect("recorded plan");
    assert!(recorded_text.contains("\"touched\""), "{recorded_text}");
}

/// AC#1, AC#2: `prepare` stages the changelog Convco generates, byte for byte, and the manifest
/// literal it planned. Nothing in pixi-release writes release-note text (`decision-6`).
#[rstest]
fn prepare_stages_the_changelog_convco_generates(cli_bin: &str) {
    let (scratch, repo) = release_repo();
    let plan = scratch.path().join("plan.json");
    write_plan(cli_bin, &repo, &plan);

    let output = Command::new(cli_bin)
        .args(["prepare", "--plan", plan.to_str().expect("utf-8 path")])
        .current_dir(&repo)
        .output()
        .expect("binary should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let generated = Command::new("convco")
        .args(["changelog", "--unreleased", "0.2.0"])
        .current_dir(&repo)
        .output()
        .expect("convco should run");
    assert!(generated.status.success());
    assert_eq!(
        std::fs::read_to_string(repo.join("CHANGELOG.md")).expect("changelog"),
        String::from_utf8_lossy(&generated.stdout),
        "the staged changelog must be Convco's own bytes"
    );
    assert!(
        std::fs::read_to_string(repo.join("Cargo.toml"))
            .expect("manifest")
            .contains("version = \"0.2.0\""),
        "the planned literal must be staged"
    );
}

/// AC#3: a changelog path the policy does not name is refused by key, naming both paths, and
/// nothing is staged — not even the manifest literals.
#[rstest]
fn prepare_refuses_a_changelog_path_the_policy_does_not_name(cli_bin: &str) {
    let (scratch, repo) = release_repo();
    let plan = scratch.path().join("plan.json");
    write_plan(cli_bin, &repo, &plan);
    let changelog_before = std::fs::read_to_string(repo.join("CHANGELOG.md")).expect("changelog");

    let output = Command::new(cli_bin)
        .args([
            "prepare",
            "--plan",
            plan.to_str().expect("utf-8 path"),
            "--changelog",
            "docs/CHANGELOG.md",
        ])
        .current_dir(&repo)
        .output()
        .expect("binary should run");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("version.changelog"), "{stderr}");
    assert!(stderr.contains("docs/CHANGELOG.md"), "{stderr}");
    assert_eq!(
        std::fs::read_to_string(repo.join("CHANGELOG.md")).expect("changelog"),
        changelog_before,
        "the changelog was not rewritten"
    );
    assert!(
        std::fs::read_to_string(repo.join("Cargo.toml"))
            .expect("manifest")
            .contains("version = \"0.1.0\""),
        "the manifest was not staged either"
    );
}

/// AC#2: a change the plan did not record is refused before anything is staged.
#[rstest]
fn cut_refuses_a_change_outside_the_recorded_set(cli_bin: &str) {
    let (scratch, repo) = release_repo();
    let plan = scratch.path().join("plan.json");
    let recorded = scratch.path().join("prepared.json");
    write_plan(cli_bin, &repo, &plan);
    prepared_plan(cli_bin, &repo, &plan, &recorded);
    std::fs::write(repo.join("notes.md"), "unrecorded\n").expect("a change the plan never saw");

    let output = Command::new(cli_bin)
        .args(["cut", "--plan", recorded.to_str().expect("utf-8 path")])
        .current_dir(&repo)
        .output()
        .expect("binary should run");

    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("worktree"),
        "the refusal names the fact"
    );
}

/// AC#2: a plan whose commit is no longer `HEAD` is stale, and is refused.
#[rstest]
fn cut_refuses_a_stale_plan(cli_bin: &str) {
    let (scratch, repo) = release_repo();
    let plan = scratch.path().join("plan.json");
    let recorded = scratch.path().join("prepared.json");
    write_plan(cli_bin, &repo, &plan);
    prepared_plan(cli_bin, &repo, &plan, &recorded);
    std::fs::write(repo.join("notes.md"), "later\n").expect("new file");
    git(&repo, &["add", "--all"]);
    git(&repo, &["commit", "--quiet", "--message", "docs: later"]);

    let output = Command::new(cli_bin)
        .args(["cut", "--plan", recorded.to_str().expect("utf-8 path")])
        .current_dir(&repo)
        .output()
        .expect("binary should run");

    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("plan.source"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// AC#3: the cut creates the release commit and tag, and writes nothing else.
#[rstest]
fn cut_commits_and_tags_exactly_the_recorded_set(cli_bin: &str) {
    let (scratch, repo) = release_repo();
    let plan = scratch.path().join("plan.json");
    let recorded = scratch.path().join("prepared.json");
    write_plan(cli_bin, &repo, &plan);
    prepared_plan(cli_bin, &repo, &plan, &recorded);
    // What a real prepare does to the tree: the recorded literal, and nothing else. REL-4 ships
    // the dry run, so the test applies the one change the dry run reported.
    std::fs::write(
        repo.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.2.0\"\n",
    )
    .expect("apply the reported edit");
    let head_before = git(&repo, &["rev-parse", "HEAD"]);

    let output = Command::new(cli_bin)
        .args(["cut", "--plan", recorded.to_str().expect("utf-8 path")])
        .current_dir(&repo)
        .output()
        .expect("binary should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let head_after = git(&repo, &["rev-parse", "HEAD"]);
    assert_ne!(head_after, head_before, "a release commit was created");
    assert_eq!(git(&repo, &["tag", "--list"]), "v0.2.0");
    let tag_message = git(&repo, &["tag", "--list", "--format=%(contents)", "v0.2.0"]);
    assert!(
        tag_message.contains("plan-digest: sha256:"),
        "{tag_message}"
    );
    assert!(
        tag_message.contains("policy-digest: sha256:"),
        "{tag_message}"
    );
    let committed = git(&repo, &["show", "--name-only", "--format=", "HEAD"]);
    let mut paths: Vec<&str> = committed.lines().filter(|line| !line.is_empty()).collect();
    paths.sort_unstable();
    assert_eq!(paths, ["Cargo.toml"]);
    assert_eq!(
        git(&repo, &["log", "--oneline", "-1", "--format=%s"]),
        "chore(release): v0.2.0"
    );
}

/// `cut --dry-run` validates everything and writes nothing.
#[rstest]
fn cut_dry_run_mutates_nothing(cli_bin: &str) {
    let (scratch, repo) = release_repo();
    let plan = scratch.path().join("plan.json");
    let recorded = scratch.path().join("prepared.json");
    write_plan(cli_bin, &repo, &plan);
    prepared_plan(cli_bin, &repo, &plan, &recorded);
    // The recorded edit, applied as a real prepare would: a dry run validates the same state.
    std::fs::write(
        repo.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.2.0\"\n",
    )
    .expect("apply the reported edit");
    let head_before = git(&repo, &["rev-parse", "HEAD"]);

    let output = Command::new(cli_bin)
        .args([
            "cut",
            "--plan",
            recorded.to_str().expect("utf-8 path"),
            "--dry-run",
        ])
        .current_dir(&repo)
        .output()
        .expect("binary should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("dry run"));
    assert_eq!(git(&repo, &["rev-parse", "HEAD"]), head_before);
    assert_eq!(git(&repo, &["tag", "--list"]), "");
    assert_eq!(
        std::fs::read_to_string(repo.join("Cargo.toml")).expect("manifest"),
        "[package]\nname = \"demo\"\nversion = \"0.2.0\"\n",
        "the dry run did not touch the manifest"
    );
}

/// Runs `prepare --dry-run --out` and returns the recorded plan path.
fn prepared_plan(cli_bin: &str, repo: &Path, plan: &Path, recorded: &Path) {
    let output = Command::new(cli_bin)
        .args([
            "prepare",
            "--plan",
            plan.to_str().expect("utf-8 path"),
            "--dry-run",
            "--out",
            recorded.to_str().expect("utf-8 path"),
        ])
        .current_dir(repo)
        .output()
        .expect("binary should run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
