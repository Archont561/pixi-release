//! Dry-run preparation: what a prepare would change, computed without changing it.
//!
//! The manifest contents are inputs, so this module never opens a file. REL-4 ships the dry run
//! only; writing the literals and regenerating the changelog land with REL-8, which is why the
//! records here are `from`/`to` pairs rather than patched documents.

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

/// Finds the first version literal in a TOML or JSON document.
///
/// TOML spells it `version = "x"`; JSON spells it `"version": "x"`. A `version.workspace = true`
/// line is not a literal, so it is skipped rather than misread.
fn version_literal(text: &str) -> Option<String> {
    for line in text.lines() {
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
        let rest = rest.trim_start();
        let rest = rest.strip_prefix('"')?;
        let (literal, _) = rest.split_once('"')?;
        if literal.is_empty() {
            return None;
        }
        return Some(literal.to_owned());
    }
    None
}
