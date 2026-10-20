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
