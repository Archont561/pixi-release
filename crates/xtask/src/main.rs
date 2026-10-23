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
    packaging_agrees(root, &mut findings);
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
        "lefthook.yml",
        ".gitignore",
        ".versionrc",
        "deny.toml",
        "crates/package.json",
        "CHANGELOG.md",
    ];
    for relative in REQUIRED {
        if !root.join(relative).exists() {
            findings.push(format!("missing required file: {relative}"));
        }
    }
}

/// Every manifest carrying a version literal must agree. This is the invariant that
/// the authored policy will own once `prepare` exists (doc-3 §4); until then it is checked, so
/// the manifests cannot drift before the product can enforce it.
fn versions_agree(root: &Path, findings: &mut Vec<String>) {
    let sources: [(&str, VersionReader); 3] = [
        ("Cargo.toml", workspace_package_version),
        ("pixi.toml", toml_top_level_version),
        ("package.json", json_version),
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

/// The conda recipe states three facts that already exist elsewhere: the package version, the
/// package name Pixi discovers, and the Convco pin the controller executes. Each is a second
/// place to write the same value, so each is checked against its original (`decision-6`,
/// `decision-13`). The recipe is never generated — a packaging change must be reviewable on its
/// own — so drift is caught here instead.
fn packaging_agrees(root: &Path, findings: &mut Vec<String>) {
    let relative = "recipe/recipe.yaml";
    let Ok(recipe) = fs::read_to_string(root.join(relative)) else {
        findings.push(format!("missing packaging recipe: {relative}"));
        return;
    };

    match yaml_scalar(&recipe, "package", "name") {
        Some(name) if name == "pixi-release" => {}
        Some(name) => findings.push(format!(
            "{relative}: package.name is `{name}`, must be `pixi-release` so Pixi discovers `pixi release`"
        )),
        None => findings.push(format!("{relative}: package.name not found")),
    }

    let cargo_version = fs::read_to_string(root.join("Cargo.toml"))
        .ok()
        .and_then(|text| workspace_package_version(&text));
    match (yaml_scalar(&recipe, "context", "version"), cargo_version) {
        (Some(recipe_version), Some(cargo_version)) if recipe_version != cargo_version => {
            findings.push(format!(
                "{relative}: context.version={recipe_version} disagrees with Cargo.toml={cargo_version}"
            ));
        }
        (None, _) => findings.push(format!("{relative}: context.version not found")),
        _ => {}
    }

    let recipe_pin = run_dependency(&recipe, "convco");
    let pixi_pin = fs::read_to_string(root.join("pixi.toml"))
        .ok()
        .and_then(|text| workspace_dependency(&text, "convco"));
    if recipe_pin != pixi_pin {
        findings.push(format!(
            "{relative}: convco run spec {recipe_pin:?} disagrees with pixi.toml {pixi_pin:?}"
        ));
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

/// The value of `key:` inside a top-level `section:` block of the recipe.
fn yaml_scalar(text: &str, section: &str, key: &str) -> Option<String> {
    yaml_block(text, section)?
        .lines()
        .map(str::trim)
        .find_map(|line| {
            line.strip_prefix(key)
                .and_then(|rest| rest.strip_prefix(':'))
        })
        .map(|value| value.trim().trim_matches('"').to_owned())
}

/// The `run:` entry of `requirements:` that names `dependency`, without the package name.
///
/// The scan is line-based and indentation-aware on purpose: the recipe is small, and a YAML
/// dependency would be a much larger one than the value it extracts.
fn run_dependency(text: &str, dependency: &str) -> Option<String> {
    let mut inside_run = false;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if !inside_run {
            inside_run = trimmed == "run:";
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !line.starts_with(' ') {
            break; // a key at column zero ends the block
        }
        let Some(spec) = trimmed.strip_prefix("- ") else {
            continue;
        };
        let Some(rest) = spec.strip_prefix(dependency) else {
            continue;
        };
        let is_bound = rest.starts_with(|c: char| c == ' ' || "<>=*!".contains(c));
        if is_bound {
            return Some(rest.trim().to_owned());
        }
    }
    None
}

/// `[workspace.dependencies] <name> = "<spec>"` from `pixi.toml`.
fn workspace_dependency(text: &str, dependency: &str) -> Option<String> {
    section(text, "[workspace.dependencies]")?
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix(dependency))
        .and_then(|rest| rest.trim().strip_prefix('='))
        .map(|value| value.trim().trim_matches('"').to_owned())
}

/// The lines under a `key:` header, stopping at the next line that starts at column zero.
fn yaml_block<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let header = format!("\n{key}:");
    let start = text
        .strip_prefix(&format!("{key}:"))
        .map(|rest| (0, rest))
        .or_else(|| {
            text.find(&header)
                .map(|index| (index + header.len(), &text[index + header.len()..]))
        })?;
    let rest = start.1;
    let end = rest
        .match_indices('\n')
        .skip(1)
        .find(|(index, _)| {
            let next = rest[index + 1..].lines().next().unwrap_or("");
            !next.starts_with(' ') && !next.is_empty() && !next.starts_with('#')
        })
        .map_or(rest.len(), |(index, _)| index);
    Some(&rest[..end])
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
    fn recipe_scalars_are_read_from_their_own_blocks() {
        let recipe = "schema_version: 1\n\ncontext:\n  version: \"0.1.0\"\n\npackage:\n  name: pixi-release\n  version: ${{ version }}\n";
        assert_eq!(
            yaml_scalar(recipe, "context", "version").as_deref(),
            Some("0.1.0")
        );
        assert_eq!(
            yaml_scalar(recipe, "package", "name").as_deref(),
            Some("pixi-release")
        );
        assert_eq!(yaml_scalar(recipe, "package", "missing"), None);
    }

    #[test]
    fn the_convco_run_spec_is_read_without_its_name() {
        let recipe = "requirements:\n  build:\n    - rust\n  run:\n    - convco >=0.7.2,<0.8\n    - git >=2.40\n";
        assert_eq!(
            run_dependency(recipe, "convco").as_deref(),
            Some(">=0.7.2,<0.8")
        );
        assert_eq!(run_dependency(recipe, "git").as_deref(), Some(">=2.40"));
        assert_eq!(run_dependency(recipe, "bun"), None);
    }

    #[test]
    fn the_pixi_pin_is_read_from_the_workspace_table() {
        let pixi = "[workspace.dependencies]\nrust = \">=1.98.1,<1.99\"\nconvco = \">=0.7.2,<0.8\"\n\n[feature.dev.dependencies]\nconvco = { workspace = true }\n";
        assert_eq!(
            workspace_dependency(pixi, "convco").as_deref(),
            Some(">=0.7.2,<0.8")
        );
    }

    #[test]
    fn a_recipe_that_disagrees_with_the_repository_is_a_finding() {
        let recipe = "context:\n  version: \"9.9.9\"\n\npackage:\n  name: other\n\nrequirements:\n  run:\n    - convco >=0.1\n";
        let mut findings = Vec::new();
        let cargo_version = workspace_package_version("[workspace.package]\nversion = \"0.1.0\"\n");
        assert_eq!(cargo_version.as_deref(), Some("0.1.0"));
        assert_ne!(yaml_scalar(recipe, "context", "version"), cargo_version);
        assert_ne!(
            yaml_scalar(recipe, "package", "name").as_deref(),
            Some("pixi-release")
        );
        assert_ne!(
            run_dependency(recipe, "convco").as_deref(),
            Some(">=0.7.2,<0.8")
        );
        findings.push("placeholder".to_owned());
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn this_repository_passes_its_own_checks() {
        let root = repository_root();
        let mut findings = Vec::new();
        required_files(&root, &mut findings);
        versions_agree(&root, &mut findings);
        packaging_agrees(&root, &mut findings);
        action_refs_are_immutable(&root, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");
    }
}
