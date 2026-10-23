//! Reading the authored policy from disk for the command line.
//!
//! Parsing and every rule live in `pixi_release_core::config`; this module only performs the I/O
//! the library deliberately refuses, and turns the surrounding files into evidence.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::process::Command;

use pixi_release_core::config::{
    ConvcoEvidence, PolicyContext, ReleasePolicy, TargetPolicy, VersionSource,
};
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

/// Renders the evidence line a plan records.
#[must_use]
pub fn render_evidence(
    plan: &pixi_release_core::config::PlanEvidence,
    policy: &ReleasePolicy,
) -> String {
    let targets: Vec<&str> = policy.targets().iter().map(TargetPolicy::name).collect();
    let mut line = format!(
        "plan: schema={} version-source={} tag-format={} targets={}",
        plan.schema(),
        match plan.version_source() {
            VersionSource::Convco => "convco",
            VersionSource::Manifest => "manifest",
        },
        policy.tag_format(),
        targets.join(",")
    );
    if let Some(convco) = plan.convco() {
        let hashes: Vec<String> = convco
            .config_hashes()
            .iter()
            .map(|(name, digest)| format!("{name}={digest}"))
            .collect();
        let _ = write!(
            line,
            " convco-version={} convco-config={}",
            convco.version(),
            hashes.join(",")
        );
    }
    line
}
