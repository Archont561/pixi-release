//! The repository facts a plan records.
//!
//! `plan` never mutates Git, so these are the only two questions it asks.

use std::process::Command;

/// Reads the commit `HEAD` points at.
///
/// # Errors
///
/// Returns a message when Git cannot run, the directory is not a repository, or the output is not
/// an object id.
pub fn head_sha() -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|error| format!("plan: git is unavailable ({error})"))?;
    if !output.status.success() {
        return Err("plan: `git rev-parse HEAD` failed; is this a repository?".to_owned());
    }
    let sha = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if sha.is_empty() {
        return Err("plan: `git rev-parse HEAD` printed nothing".to_owned());
    }
    Ok(sha)
}

/// Reports whether the worktree is clean.
///
/// # Errors
///
/// Returns a message when Git cannot run or the directory is not a repository.
pub fn is_clean() -> Result<bool, String> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map_err(|error| format!("plan: git is unavailable ({error})"))?;
    if !output.status.success() {
        return Err("plan: `git status --porcelain` failed; is this a repository?".to_owned());
    }
    Ok(output.stdout.is_empty())
}
