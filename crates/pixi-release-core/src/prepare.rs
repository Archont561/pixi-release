//! Preparation as data: what a prepare changes, computed before anything is written.
//!
//! The manifest contents are inputs, so this module never opens a file. An `Edit` is a `from`/`to`
//! record, `stage` turns it into the document it describes, and `changelog_target` decides which
//! path may receive Convco's output — the CLI performs the I/O and never the text (`decision-6`).

use std::fmt;

use crate::plan::Version;

/// One intended change: a manifest and the literal it carries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Edit {
    path: String,
    from: String,
    to: String,
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
        });
    }
    Ok(edits)
}

/// Writes `edit` into `text`, replacing one literal and leaving the rest of the document alone.
///
/// The line is chosen by the same scan that found the literal, so a writer and a reader can never
/// disagree about which `version` a manifest carries. When the manifest no longer holds the
/// literal the edit was computed from, the text comes back unchanged rather than half-rewritten:
/// `prepare` recomputes its edits from the files it just read, so this is a consistency guard.
#[must_use]
pub fn stage(text: &str, edit: &Edit) -> String {
    let Some((line_number, _)) = version_value(text) else {
        return text.to_owned();
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
