use std::process::Command;

#[test]
fn version_is_a_stable_cli_surface() {
    let output = Command::new(env!("CARGO_BIN_EXE_pixi-release"))
        .arg("--version")
        .output()
        .expect("scaffold binary should run");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "pixi-release 0.1.0\n"
    );
}

#[test]
fn unknown_command_is_refused() {
    let output = Command::new(env!("CARGO_BIN_EXE_pixi-release"))
        .arg("unknown")
        .output()
        .expect("scaffold binary should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("pixi-release <"));
}
