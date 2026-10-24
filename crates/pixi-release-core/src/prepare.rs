//! Preparation as data: what a prepare changes, computed before anything is written.
//!
//! The manifest contents are inputs, so this module never opens a file. An `Edit` is a `from`/`to`
//! record, `stage` turns it into the document it describes, and `changelog_target` decides which
//! path may receive Convco's output — the CLI performs the I/O and never the text (`decision-6`).
//!
//! A prepare is a sequence of **stages**, and the sequence is data too ([`STAGES`]): the literals a
//! manifest carries, then the internal pins that must move with them, then the generated files that
//! read the manifest, then the lockfiles that record the result, and the changelog last. The order
//! cannot be a comment, because every step after the first reads what the step before wrote — a
//! renderer that runs before the stamp writes the previous version into the release (pixi-sandbox
//! v0.5.3), and a lockfile refreshed before the manifests keeps the old one (REL-29).

use std::fmt;

use crate::plan::Version;

/// One intended change: a manifest and the literal it carries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Edit {
    path: String,
    from: String,
    to: String,
    /// The line the literal sits on, when the producer knows it. `None` means "find it with the
    /// same scan that found the literal", which is what the manifest stage does.
    line: Option<usize>,
}

impl Edit {
    /// The manifest whose literal would change.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The literal it carries now.
    #[must_use]
    pub fn from(&self) -> &str {
        &self.from
    }

    /// The literal the plan asks for.
    #[must_use]
    pub fn to(&self) -> &str {
        &self.to
    }

    /// The line the literal sits on, when the producer recorded one.
    #[must_use]
    pub fn line(&self) -> Option<usize> {
        self.line
    }
}

/// The steps a prepare applies, in the only order that cannot contradict itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stage {
    /// The version literals the policy names.
    Manifests,
    /// Path dependencies that pin a workspace member at a literal version.
    Pins,
    /// Generated files, which read the stamped manifest when they are rendered.
    Generated,
    /// Lockfiles, which record what the manifests just said.
    Lockfiles,
    /// Convco's changelog for the version being cut (`decision-6`).
    Changelog,
}

/// The canonical stage order. Every stage reads what the one before it wrote, so this list is the
/// contract: `Generated` after `Manifests` (a renderer that runs first stamps the previous
/// version), `Lockfiles` after both (a lock refreshed early keeps the old members), and
/// `Changelog` last (it is the only stage that does not feed another).
pub const STAGES: [Stage; 5] = [
    Stage::Manifests,
    Stage::Pins,
    Stage::Generated,
    Stage::Lockfiles,
    Stage::Changelog,
];

/// The stage order a caller must apply. Exposed as data so a test can pin it, rather than as a
/// comment a reordering would leave true.
#[must_use]
pub fn stages() -> &'static [Stage] {
    &STAGES
}

/// A refusal to describe a preparation, naming the manifest at fault.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrepareRefusal {
    key: String,
    detail: String,
}

impl PrepareRefusal {
    fn new(key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            detail: detail.into(),
        }
    }

    /// The key at fault.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Why the preparation was refused.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for PrepareRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.key, self.detail)
    }
}

impl std::error::Error for PrepareRefusal {}

/// The changelog path a prepare may stage, refusing any path the policy does not name.
///
/// `requested` is what the caller asked for; `None` means the policy's own path. The comparison
/// ignores surrounding whitespace and nothing else: a near miss is a different file, and staging
/// the wrong one would put Convco's text where `cut` does not look for it (`decision-6`).
///
/// # Errors
///
/// Returns `version.changelog` when the policy names no path, or when the request names another.
pub fn changelog_target(
    policy_changelog: &str,
    requested: Option<&str>,
) -> Result<String, PrepareRefusal> {
    let named = policy_changelog.trim();
    if named.is_empty() {
        return Err(PrepareRefusal::new(
            "version.changelog",
            "the policy names no changelog path; Convco writes one (decision-6)",
        ));
    }
    match requested.map(str::trim).filter(|path| !path.is_empty()) {
        Some(requested) if requested != named => Err(PrepareRefusal::new(
            "version.changelog",
            format!("`{requested}` is not the changelog path this policy names (`{named}`)"),
        )),
        _ => Ok(named.to_owned()),
    }
}

/// Reports the version literals a prepare would change, in the order the policy listed them.
///
/// A manifest already at `target` needs no edit and is not recorded; a manifest with no readable
/// literal is refused, because "the policy listed it" and "we cannot see its version" cannot both
/// be true.
///
/// # Errors
///
/// Returns `version.manifests` naming the manifest whose literal could not be read.
pub fn version_edits<I>(manifests: I, target: &Version) -> Result<Vec<Edit>, PrepareRefusal>
where
    I: IntoIterator<Item = (String, String)>,
{
    let mut edits = Vec::new();
    for (path, text) in manifests {
        let Some(current) = version_literal(&text) else {
            return Err(PrepareRefusal::new(
                "version.manifests",
                format!("`{path}` carries no version literal to update"),
            ));
        };
        if current == target.as_str() {
            continue;
        }
        edits.push(Edit {
            path,
            from: current,
            to: target.as_str().to_owned(),
            line: None,
        });
    }
    Ok(edits)
}

/// Reports the internal pins a prepare must move: path dependencies that also carry a literal
/// version, inside a Cargo `[workspace.dependencies]` table.
///
/// Cargo resolves a `path` dependency by the path alone, so the literal beside it looks decorative
/// — and is not. `cargo-deny` treats an unpinned workspace dependency as a wildcard, and a member
/// whose pin lags the workspace version is a published package claiming to depend on a release that
/// does not exist. `[workspace.dependencies]` is the one table where the two spellings meet, so the
/// scan is anchored on the table header rather than on a line that happens to mention `version`:
/// `.dependencies` of a *crate* name their requirements, not our own version.
///
/// A manifest that is not a Cargo manifest has no pins, and a table entry with no `version` has
/// none to move; neither is an error. An entry that has both and whose literal cannot be read is
/// refused, because "the pin exists" and "its value is unreadable" cannot both be true.
///
/// # Errors
///
/// Returns `version.manifests` naming the Cargo manifest and the line whose pin is unreadable.
pub fn pin_edits<I>(manifests: I, target: &Version) -> Result<Vec<Edit>, PrepareRefusal>
where
    I: IntoIterator<Item = (String, String)>,
{
    let mut edits = Vec::new();
    for (path, text) in manifests {
        if !is_cargo_manifest(&path) {
            continue;
        }
        let mut in_workspace_dependencies = false;
        for (index, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') {
                in_workspace_dependencies = trimmed == "[workspace.dependencies]";
                continue;
            }
            if !in_workspace_dependencies || !trimmed.contains("path") {
                continue;
            }
            let Some(rest) = trimmed.split_once("version").map(|(_, rest)| rest) else {
                continue; // a path dependency with no literal has none to move
            };
            let Some(literal) = quoted_after_equals(rest) else {
                return Err(PrepareRefusal::new(
                    "version.manifests",
                    format!(
                        "`{path}`:{}: a path dependency names a version this scan cannot read, \
                         so the pin would silently stay behind",
                        index + 1
                    ),
                ));
            };
            if literal == target.as_str() {
                continue;
            }
            edits.push(Edit {
                path: path.clone(),
                from: literal,
                to: target.as_str().to_owned(),
                line: Some(index),
            });
        }
    }
    Ok(edits)
}

/// Whether the policy entry is a Cargo manifest, by file name and nothing else.
fn is_cargo_manifest(path: &str) -> bool {
    path.rsplit(['/', '\\']).next() == Some("Cargo.toml")
}

/// Reads the quoted literal that follows `=`, tolerating spaces around it.
fn quoted_after_equals(text: &str) -> Option<String> {
    let rest = text.split_once('=')?.1.trim_start();
    let rest = rest.strip_prefix('"')?;
    let (literal, _) = rest.split_once('"')?;
    (!literal.is_empty()).then(|| literal.to_owned())
}

/// Writes `edit` into `text`, replacing one literal and leaving the rest of the document alone.
///
/// The line is chosen by the same scan that found the literal, so a writer and a reader can never
/// disagree about which `version` a manifest carries. When the manifest no longer holds the
/// literal the edit was computed from, the text comes back unchanged rather than half-rewritten:
/// `prepare` recomputes its edits from the files it just read, so this is a consistency guard.
#[must_use]
pub fn stage(text: &str, edit: &Edit) -> String {
    let line_number = match edit.line() {
        Some(line) => line,
        None => match version_value(text) {
            Some((line, _)) => line,
            None => return text.to_owned(),
        },
    };
    let from = format!("\"{}\"", edit.from());
    let to = format!("\"{}\"", edit.to());
    let mut staged = String::with_capacity(text.len() + to.len());
    for (index, line) in text.split_inclusive('\n').enumerate() {
        if index == line_number && line.contains(&from) {
            staged.push_str(&line.replacen(&from, &to, 1));
        } else {
            staged.push_str(line);
        }
    }
    staged
}

/// Finds the first version literal in a TOML or JSON document.
///
/// TOML spells it `version = "x"`; JSON spells it `"version": "x"`. A `version.workspace = true`
/// line is not a literal, so it is skipped rather than misread.
fn version_literal(text: &str) -> Option<String> {
    let (_, rest) = version_value(text)?;
    let rest = rest.strip_prefix('"')?;
    let (literal, _) = rest.split_once('"')?;
    if literal.is_empty() {
        return None;
    }
    Some(literal.to_owned())
}

/// Locates the first line that carries a version key, as its line index and the text after the key.
///
/// TOML spells the key `version =`; JSON spells it `"version":`. A `version.workspace = true` line
/// is a key without a literal, so the scan stops there rather than reading a later `version` that
/// belongs to something else — the manifest is ambiguous at that point, not two manifests long.
fn version_value(text: &str) -> Option<(usize, &str)> {
    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        let rest = if let Some(rest) = line.strip_prefix("\"version\"") {
            rest.trim_start().strip_prefix(':')?
        } else if let Some(rest) = line.strip_prefix("version") {
            if !rest.starts_with([' ', '\t', '=']) {
                continue;
            }
            rest.trim_start().strip_prefix('=')?
        } else {
            continue;
        };
        return Some((index, rest.trim_start()));
    }
    None
}
