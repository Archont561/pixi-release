//! Reading the authored policy from disk for the command line.
//!
//! Parsing and every rule live in `pixi_release_core::config`; this module only performs the I/O
//! the library deliberately refuses, and turns the surrounding files into evidence.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::process::Command;

use pixi_release_core::config::{ConvcoEvidence, PolicyContext, ReleasePolicy};
use pixi_release_core::plan::Bump;
use sha2::{Digest, Sha256};

/// Locates the repository facts the parser needs. Working-directory relative by design: the
/// command is meant to run at the repository root.
#[must_use]
pub fn context() -> PolicyContext {
    PolicyContext {
        convco_configured: Path::new(".versionrc").is_file() || Path::new(".convco").is_file(),
    }
}

/// Reads and validates the policy at `path`.
///
/// # Errors
///
/// Returns the rendered refusals when the file is unreadable or the policy is invalid.
pub fn load(path: &str, context: PolicyContext) -> Result<ReleasePolicy, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("policy: {path}: {error}"))?;
    ReleasePolicy::parse_with(&text, &context).map_err(|errors| {
        let mut rendered = String::new();
        for error in errors.iter() {
            let _ = writeln!(rendered, "policy: {error}");
        }
        rendered.trim_end().to_owned()
    })
}

/// The `sha256:` digest of the policy file the plan was built under (`decision-3`).
///
/// # Errors
///
/// Returns a message when the file cannot be read.
pub fn digest(path: &str) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("policy: {path}: {error}"))?;
    Ok(format!("sha256:{}", sha256_hex(&bytes)))
}

/// Collects Convco evidence: its version, and the digests of the configuration it read.
///
/// # Errors
///
/// Returns a message when no version can be established or no configuration file is present.
pub fn convco_evidence(version: Option<&str>) -> Result<ConvcoEvidence, String> {
    let version = match version {
        Some(version) => version.to_owned(),
        None => installed_convco_version()?,
    };
    let mut hashes = BTreeMap::new();
    for name in [".versionrc", ".convco"] {
        if let Ok(bytes) = fs::read(name) {
            hashes.insert(name.to_owned(), sha256_hex(&bytes));
        }
    }
    ConvcoEvidence::new(version, hashes).map_err(|error| format!("plan: {error}"))
}

/// Asks Convco for the next version, so the plan never derives one itself (`decision-6`).
///
/// # Errors
///
/// Returns Convco's own message when the tool is missing or refuses to answer — a repository with
/// no releasable commits is a refusal, not a fallback.
pub fn convco_next_version(bump: Bump) -> Result<String, String> {
    let mut command = Command::new("convco");
    command.arg("version").arg("--bump");
    match bump {
        Bump::Auto => {}
        Bump::Major => {
            command.arg("--major");
        }
        Bump::Minor => {
            command.arg("--minor");
        }
        Bump::Patch => {
            command.arg("--patch");
        }
    }
    let output = command.output().map_err(|error| {
        format!("plan: convco is unavailable ({error}); it derives the version (decision-6)")
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("plan: convco refused ({})", stderr.trim()));
    }
    let version = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if version.is_empty() {
        return Err("plan: convco printed no version".to_owned());
    }
    Ok(version)
}

/// Asks Convco to write the changelog for `version`, and returns its bytes untouched.
///
/// Capture rather than `convco changelog --output`: the dry run can then report what would be
/// staged without creating the file, and the write stays a single atomic call. The text is still
/// Convco's — pixi-release contributes no release-note text of its own (`decision-6`).
///
/// # Errors
///
/// Returns a message when Convco is missing, refuses, or produces nothing.
pub fn convco_changelog(version: &str) -> Result<Vec<u8>, String> {
    let output = Command::new("convco")
        .arg("changelog")
        .arg("--unreleased")
        .arg(version)
        .output()
        .map_err(|error| {
            format!(
                "prepare: convco is unavailable ({error}); it writes the changelog (decision-6)"
            )
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("prepare: convco refused ({})", stderr.trim()));
    }
    if output.stdout.is_empty() {
        return Err(format!(
            "prepare: convco produced no changelog for {version}"
        ));
    }
    Ok(output.stdout)
}

fn installed_convco_version() -> Result<String, String> {
    let output = Command::new("convco")
        .arg("--version")
        .output()
        .map_err(|error| {
            format!(
                "plan: convco is unavailable ({error}); pass --convco-version or install convco \
                 (decision-6)"
            )
        })?;
    if !output.status.success() {
        return Err("plan: `convco --version` failed; pass --convco-version".to_owned());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let version = stdout.split_whitespace().last().unwrap_or_default();
    if version.is_empty() {
        return Err("plan: `convco --version` printed no version".to_owned());
    }
    Ok(version.to_owned())
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut rendered, byte| {
            let _ = write!(rendered, "{byte:02x}");
            rendered
        })
}
