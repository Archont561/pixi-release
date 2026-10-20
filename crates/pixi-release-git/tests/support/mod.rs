//! Shared dummy-repository builders for pixi-release tests.
//!
//! Build dummy repositories through these plain functions. Tests receive them through rstest
//! fixtures that wrap the builder, so each fixture case gets a fresh repository.

use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

/// A committed, clean dummy repository that is removed when dropped.
pub struct DummyRepo {
    _dir: TempDir,
    path: PathBuf,
}

impl DummyRepo {
    /// Root of the dummy repository.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Runs `git` in `dir` and returns trimmed stdout, panicking with stderr on failure.
#[must_use]
pub fn git_output(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("git should run");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

/// Builds a dummy repository with one committed `pixi.toml` and a clean worktree.
#[must_use]
pub fn build_dummy_repo(name: &str) -> DummyRepo {
    let dir = tempfile::Builder::new()
        .prefix(&format!("pixi-release-{name}-"))
        .tempdir()
        .expect("temp dir should be created");
    let path = dir.path().to_path_buf();

    git_output(&path, &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(path.join("pixi.toml"), "[workspace]\nname = \"dummy\"\n")
        .expect("pixi.toml should be written");
    git_output(&path, &["add", "pixi.toml"]);
    git_output(
        &path,
        &[
            "-c",
            "user.name=pixi-release-tests",
            "-c",
            "user.email=tests@example.invalid",
            "commit",
            "--quiet",
            "--message",
            "Initial dummy release candidate",
        ],
    );

    DummyRepo { _dir: dir, path }
}
