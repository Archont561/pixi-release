//! Owner-lane repository automation.
//!
//! This crate is **not** the product. `pixi-release` (the product) cannot be used to cut
//! `pixi-release` v0.1.0, and `AGENTS.md` forbids the alternative of executing the unreleased
//! checkout as its own controller. `xtask` is what cuts v0.1.0.
//!
//! Deletion trigger, from `doc-3` §1.3: **xtask's release subcommands are deleted once
//! `consumer-proof` has proven a pinned prior `pixi-release` cutting this repository's release
//! end to end.** Without a written trigger this becomes a permanent second release authority,
//! which is precisely the second command path `AGENTS.md` forbids. Until that day arrives,
//! xtask deliberately ships repository checks only — no release subcommands exist yet.
//!
//! Intentionally dependency-free: the crate that checks the lockfile should not enlarge it.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

const USAGE: &str = "xtask <check>";

/// Reads a version literal out of one manifest dialect.
type VersionReader = fn(&str) -> Option<String>;

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let root = repository_root();
    let exit_code = if arguments.first().map(String::as_str) == Some("check") {
        run_checks(&root)
    } else {
        eprintln!("{USAGE}");
        2
    };
    std::process::exit(exit_code);
}

/// `CARGO_MANIFEST_DIR` is `<root>/crates/xtask`, so the root is two levels up. This avoids
/// depending on the caller's working directory, which differs between pixi, Turbo and a hook.
fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xtask lives at <root>/crates/xtask")
        .to_path_buf()
}

fn run_checks(root: &Path) -> i32 {
    let mut findings: Vec<String> = Vec::new();

    required_files(root, &mut findings);
    versions_agree(root, &mut findings);
    action_refs_are_immutable(root, &mut findings);

    if findings.is_empty() {
        println!("xtask check: repository checks passed");
        return 0;
    }
    for finding in &findings {
        eprintln!("::error::{finding}");
    }
    eprintln!("xtask check: {} finding(s)", findings.len());
    1
}

/// The workspace contract. `crates/xtask/scripts/verify-workspace.ts` asserts the JS half; this asserts the
/// files a release depends on existing at all.
fn required_files(root: &Path, findings: &mut Vec<String>) {
    const REQUIRED: &[&str] = &[
        "Cargo.toml",
        "pixi.toml",
        "package.json",
        "turbo.json",
        "biome.json",
        "lefthook.yml",
        ".gitignore",
        ".versionrc",
        ".pixi-release.toml",
        "deny.toml",
        "crates/package.json",
        "apps/docs/package.json",
        "CHANGELOG.md",
    ];
    for relative in REQUIRED {
        if !root.join(relative).exists() {
            findings.push(format!("missing required file: {relative}"));
        }
    }
}

/// Every manifest carrying a version literal must agree. This is the invariant that
/// `.pixi-release.toml`'s `[version].manifests` list will own once `prepare` exists; until
/// then it is checked, so the two cannot drift before the product can enforce it.
fn versions_agree(root: &Path, findings: &mut Vec<String>) {
    let sources: [(&str, VersionReader); 4] = [
        ("Cargo.toml", workspace_package_version),
        ("pixi.toml", toml_top_level_version),
        ("package.json", json_version),
        ("apps/docs/package.json", json_version),
    ];

    let mut seen: Vec<(&str, String)> = Vec::new();
    for (relative, extract) in sources {
        let Ok(text) = fs::read_to_string(root.join(relative)) else {
            continue; // absence is already reported by required_files
        };
        match extract(&text) {
            Some(version) => seen.push((relative, version)),
            None => findings.push(format!("{relative}: no version literal found")),
        }
    }

    let Some((_, first)) = seen.first() else {
        return;
    };
    if seen.iter().any(|(_, version)| version != first) {
        let mut detail = String::new();
        for (relative, version) in &seen {
            let _ = write!(detail, " {relative}={version}");
        }
        findings.push(format!("manifest versions disagree:{detail}"));
    }
}

/// A mutable action ref is a supply-chain hole and the family already pins by full SHA
/// (`doc-4` §7b). Local `./` actions are exempt; they are in this repository and reviewed here.
fn action_refs_are_immutable(root: &Path, findings: &mut Vec<String>) {
    let workflows = root.join(".github/workflows");
    let Ok(entries) = fs::read_dir(&workflows) else {
        return; // no owner lane yet
    };
    let mut paths: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("yml" | "yaml")
            )
        })
        .collect();
    paths.sort();

    for path in paths {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        for (index, line) in text.lines().enumerate() {
            let Some(reference) = uses_value(line) else {
                continue;
            };
            if reference.starts_with("./") || reference.starts_with("docker://") {
                continue;
            }
            let Some((_, git_ref)) = reference.rsplit_once('@') else {
                findings.push(format!(
                    "{name}:{}: `uses:` has no ref: {reference}",
                    index + 1
                ));
                continue;
            };
            if !is_full_sha(git_ref) {
                findings.push(format!(
                    "{name}:{}: mutable action ref `{git_ref}` — pin the full commit SHA",
                    index + 1
                ));
            }
        }
    }
}

fn uses_value(line: &str) -> Option<&str> {
    let trimmed = line
        .trim_start()
        .strip_prefix("- ")
        .unwrap_or(line.trim_start());
    let rest = trimmed.strip_prefix("uses:")?;
    let value = rest.split('#').next().unwrap_or("").trim();
    (!value.is_empty()).then_some(value)
}

fn is_full_sha(value: &str) -> bool {
    value.len() == 40 && value.chars().all(|c| c.is_ascii_hexdigit())
}

// --- minimal manifest readers -------------------------------------------------------------
// Deliberately naive: these read our own four files, whose shape is fixed and reviewed.

fn workspace_package_version(text: &str) -> Option<String> {
    section(text, "[workspace.package]").and_then(bare_version)
}

fn toml_top_level_version(text: &str) -> Option<String> {
    section(text, "[workspace]").and_then(bare_version)
}

fn section<'a>(text: &'a str, header: &str) -> Option<&'a str> {
    let start = text.find(header)? + header.len();
    let rest = &text[start..];
    let end = rest.find("\n[").unwrap_or(rest.len());
    Some(&rest[..end])
}

fn bare_version(section: &str) -> Option<String> {
    section
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("version"))
        .and_then(|rest| rest.trim().strip_prefix('='))
        .map(|value| value.trim().trim_matches('"').to_owned())
}

fn json_version(text: &str) -> Option<String> {
    let start = text.find("\"version\"")? + "\"version\"".len();
    let rest = text[start..].trim_start().strip_prefix(':')?.trim_start();
    let inner = rest.strip_prefix('"')?;
    let end = inner.find('"')?;
    Some(inner[..end].to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_shas_are_the_only_acceptable_ref() {
        assert!(is_full_sha("3d3c42e5aac5ba805825da76410c181273ba90b1"));
        assert!(!is_full_sha("v7.0.1"));
        assert!(!is_full_sha("main"));
        assert!(!is_full_sha("3d3c42e"));
    }

    #[test]
    fn a_uses_line_is_read_without_its_version_comment() {
        let line =
            "        uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1";
        assert_eq!(
            uses_value(line),
            Some("actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1")
        );
        assert_eq!(uses_value("        run: echo uses:"), None);
    }

    #[test]
    fn versions_are_read_from_each_manifest_dialect() {
        let cargo = "[workspace]\nmembers = []\n\n[workspace.package]\nversion = \"0.1.0\"\n";
        assert_eq!(workspace_package_version(cargo).as_deref(), Some("0.1.0"));

        let pixi = "[workspace]\nname = \"pixi-release\"\nversion = \"0.1.0\"\n\n[tasks]\n";
        assert_eq!(toml_top_level_version(pixi).as_deref(), Some("0.1.0"));

        let json = "{\n  \"name\": \"x\",\n  \"version\": \"0.1.0\"\n}";
        assert_eq!(json_version(json).as_deref(), Some("0.1.0"));
    }

    #[test]
    fn this_repository_passes_its_own_checks() {
        let root = repository_root();
        let mut findings = Vec::new();
        required_files(&root, &mut findings);
        versions_agree(&root, &mut findings);
        action_refs_are_immutable(&root, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");
    }
}
