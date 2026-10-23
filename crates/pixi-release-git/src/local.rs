//! The real adapter: `git` in a subprocess, against the repository at a path.
//!
//! It implements only the two traits in the parent module, so the set of things a release run can
//! do to a repository is the set of commands spelled out here.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{GitError, GitOp, GitProtocol, ReleaseGit, TagRef};

/// A repository reached through the `git` executable.
#[derive(Clone, Debug)]
pub struct LocalGit {
    root: PathBuf,
    head_sha: String,
    dirty_paths: Vec<String>,
    branch: Option<String>,
    tags: Vec<TagRef>,
    operations: Vec<GitOp>,
}

impl LocalGit {
    /// Reads the repository state at `root` without changing anything.
    ///
    /// # Errors
    ///
    /// Returns the first command failure, so a caller never validates against a partial picture.
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, GitError> {
        let root = root.into();
        let head_sha = run(&root, &["rev-parse", "HEAD"])?;
        let branch = run(&root, &["rev-parse", "--abbrev-ref", "HEAD"])?;
        let branch = if branch == "HEAD" { None } else { Some(branch) };
        let dirty_paths = parse_status(&run(&root, &["status", "--porcelain"])?);
        let mut tags = Vec::new();
        for line in run(&root, &["tag", "--list"])?.lines() {
            let name = line.trim();
            if name.is_empty() {
                continue;
            }
            let commit = run(&root, &["rev-list", "-n", "1", name])?;
            tags.push(TagRef::new(name, commit));
        }
        Ok(Self {
            root,
            head_sha,
            dirty_paths,
            branch,
            tags,
            operations: Vec::new(),
        })
    }

    /// The repository root this adapter writes to.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl GitProtocol for LocalGit {
    fn head_sha(&self) -> &str {
        &self.head_sha
    }

    fn dirty_paths(&self) -> &[String] {
        &self.dirty_paths
    }

    fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }

    fn tags(&self) -> &[TagRef] {
        &self.tags
    }
}

impl ReleaseGit for LocalGit {
    fn stage(&mut self, paths: &[String]) -> Result<(), GitError> {
        if paths.is_empty() {
            return Err(GitError::new(
                "stage",
                "no path was staged, so nothing is recorded",
            ));
        }
        let mut arguments = vec!["add".to_owned(), "--".to_owned()];
        arguments.extend(paths.iter().cloned());
        let borrowed: Vec<&str> = arguments.iter().map(String::as_str).collect();
        run(&self.root, &borrowed)?;
        self.operations.push(GitOp::Stage(paths.to_vec()));
        self.dirty_paths = paths.to_vec();
        Ok(())
    }

    fn commit(&mut self, message: &str) -> Result<String, GitError> {
        run(&self.root, &["commit", "--message", message])?;
        self.operations.push(GitOp::Commit(message.to_owned()));
        self.head_sha = run(&self.root, &["rev-parse", "HEAD"])?;
        self.dirty_paths = parse_status(&run(&self.root, &["status", "--porcelain"])?);
        Ok(self.head_sha.clone())
    }

    fn tag(&mut self, name: &str, message: &str) -> Result<(), GitError> {
        run(
            &self.root,
            &["tag", "--annotate", name, "--message", message],
        )?;
        self.operations.push(GitOp::Tag(name.to_owned()));
        self.tags.push(TagRef::new(name, self.head_sha.clone()));
        Ok(())
    }

    fn push(&mut self, refspec: &str) -> Result<(), GitError> {
        run(&self.root, &["push", "origin", refspec])?;
        self.operations.push(GitOp::Push(refspec.to_owned()));
        Ok(())
    }

    fn operations(&self) -> &[GitOp] {
        &self.operations
    }
}

/// Turns `git status --porcelain` output into the paths it names.
///
/// Each line is `XY <path>`, with a rename reported as `old -> new`; the destination is the path
/// that exists now, so that is the one a plan must have recorded.
fn parse_status(status: &str) -> Vec<String> {
    status
        .lines()
        .filter_map(|line| {
            let path = line.get(3..)?.trim();
            if path.is_empty() {
                return None;
            }
            Some(
                path.rsplit_once(" -> ")
                    .map_or(path, |(_, destination)| destination)
                    .trim_matches('"')
                    .to_owned(),
            )
        })
        .collect()
}

/// Runs `git -C root <arguments>` and returns trimmed stdout.
fn run(root: &Path, arguments: &[&str]) -> Result<String, GitError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()
        .map_err(|error| GitError::new(arguments.join(" "), error.to_string()))?;
    if !output.status.success() {
        return Err(GitError::new(
            arguments.join(" "),
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_owned())
}
