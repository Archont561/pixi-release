use std::process::Command;

use rstest::{fixture, rstest};

#[fixture]
fn cli_bin() -> &'static str {
    env!("CARGO_BIN_EXE_pixi-release")
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

/// A policy with `replacement` applied, for the refusal cases.
fn policy_with(from: &str, to: &str) -> String {
    POLICY.replace(from, to)
}

/// AC#2: `plan` in Convco mode records the version and configuration digests as evidence.
#[rstest]
fn plan_records_convco_version_and_config_hashes(cli_bin: &str) {
    let dir = tempfile::tempdir().expect("temp dir");
    std::fs::write(dir.path().join(".pixi-release.toml"), POLICY).expect("policy");
    std::fs::write(dir.path().join(".versionrc"), "{\"types\":[]}").expect("versionrc");

    let output = Command::new(cli_bin)
        .args(["plan", "--convco-version", "0.4.2"])
        .current_dir(dir.path())
        .output()
        .expect("binary should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("version-source=convco"), "{stdout}");
    assert!(stdout.contains("convco-version=0.4.2"), "{stdout}");
    let digest = {
        use std::fmt::Write as _;

        use sha2::{Digest as _, Sha256};
        Sha256::digest(br#"{"types":[]}"#)
            .iter()
            .fold(String::new(), |mut rendered, byte| {
                let _ = write!(rendered, "{byte:02x}");
                rendered
            })
    };
    assert!(
        stdout.contains(&format!(".versionrc={digest}")),
        "expected the configuration digest in {stdout}"
    );
    assert!(stdout.contains("targets=github-release"), "{stdout}");
}

/// AC#1: an unsafe combination is refused before any I/O, naming the exact key.
#[rstest]
fn plan_refuses_convco_with_a_manifest_source(cli_bin: &str) {
    let dir = tempfile::tempdir().expect("temp dir");
    let policy = policy_with(
        "version-source = \"convco\"",
        "version-source = \"manifest\"",
    );
    std::fs::write(dir.path().join(".pixi-release.toml"), policy).expect("policy");
    std::fs::write(dir.path().join(".versionrc"), "{}").expect("versionrc");

    let output = Command::new(cli_bin)
        .args(["plan", "--convco-version", "0.4.2"])
        .current_dir(dir.path())
        .output()
        .expect("binary should run");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("project.version-source"), "{stderr}");
    assert!(stderr.contains("decision-6"), "{stderr}");
}

/// AC#3: native `SemVer` is refused while a Convco configuration is active.
#[rstest]
fn plan_refuses_native_semver_while_convco_is_active(cli_bin: &str) {
    let dir = tempfile::tempdir().expect("temp dir");
    let policy = policy_with("version-source = \"convco\"", "version-source = \"semver\"");
    std::fs::write(dir.path().join(".pixi-release.toml"), policy).expect("policy");
    std::fs::write(dir.path().join(".versionrc"), "{}").expect("versionrc");

    let output = Command::new(cli_bin)
        .arg("plan")
        .current_dir(dir.path())
        .output()
        .expect("binary should run");

    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("project.version-source"),
        "refusal must name the key"
    );
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
