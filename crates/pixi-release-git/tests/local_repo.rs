//! The real adapter against a real repository: a local bare remote, no network (`DoD` #2).

use std::path::Path;
use std::process::Command;

use pixi_release_git::{GitOp, GitProtocol, LocalGit, ReleaseGit};
use rstest::{fixture, rstest};

/// Runs `git -C <dir> <args>`, returning trimmed stdout.
fn git(dir: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(arguments)
        .output()
        .expect("git should run");
    assert!(
        output.status.success(),
        "git {arguments:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

/// A working repository with one commit and a bare `origin`.
struct Fixture {
    _root: tempfile::TempDir,
    repo: std::path::PathBuf,
    remote: std::path::PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("temp dir");
        let remote = root.path().join("remote.git");
        let repo = root.path().join("repo");
        std::fs::create_dir(&repo).expect("repo dir");

        git(
            root.path(),
            &["init", "--quiet", "--bare", remote.to_str().expect("utf-8")],
        );
        git(&repo, &["init", "--quiet", "--initial-branch", "main"]);
        git(&repo, &["config", "user.email", "test@example.com"]);
        git(&repo, &["config", "user.name", "Test"]);
        git(
            &repo,
            &["remote", "add", "origin", remote.to_str().expect("utf-8")],
        );
        std::fs::write(repo.join("Cargo.toml"), "version = \"0.1.0\"\n").expect("manifest");
        git(&repo, &["add", "--all"]);
        git(&repo, &["commit", "--quiet", "--message", "chore: init"]);

        Self {
            _root: root,
            repo,
            remote,
        }
    }
}

#[fixture]
fn fixture() -> Fixture {
    Fixture::new()
}

/// The read side reports what `git` reports.
#[rstest]
fn open_reads_the_repository_state(fixture: Fixture) {
    let observed = LocalGit::open(&fixture.repo).expect("open");

    assert_eq!(
        observed.head_sha(),
        git(&fixture.repo, &["rev-parse", "HEAD"])
    );
    assert_eq!(observed.branch(), Some("main"));
    assert!(observed.is_clean());
    assert!(observed.tags().is_empty());
}

/// A dirty tree and a detached head are both visible, so `cut` can refuse them.
#[rstest]
fn open_reports_a_dirty_tree_and_a_detached_head(fixture: Fixture) {
    std::fs::write(fixture.repo.join("Cargo.toml"), "version = \"9.9.9\"\n").expect("edit");
    let dirty = LocalGit::open(&fixture.repo).expect("open");
    assert!(!dirty.is_clean());

    git(&fixture.repo, &["checkout", "--quiet", "--", "Cargo.toml"]);
    git(&fixture.repo, &["checkout", "--quiet", "--detach", "HEAD"]);
    let detached = LocalGit::open(&fixture.repo).expect("open");
    assert_eq!(detached.branch(), None);
}

/// The mutations reach the bare remote: a commit and an annotated tag arrive where `cut` pushed them.
#[rstest]
fn stage_commit_tag_and_push_reach_the_bare_remote(fixture: Fixture) {
    let mut observed = LocalGit::open(&fixture.repo).expect("open");
    std::fs::write(fixture.repo.join("Cargo.toml"), "version = \"0.2.0\"\n").expect("edit");

    observed
        .stage(&["Cargo.toml".to_owned()])
        .expect("stage succeeds");
    let commit = observed
        .commit("chore(release): v0.2.0")
        .expect("commit succeeds");
    observed
        .tag("v0.2.0", "v0.2.0\n\nplan-digest: sha256:00")
        .expect("tag succeeds");
    observed
        .push("refs/heads/main")
        .expect("branch push succeeds");
    observed
        .push("refs/tags/v0.2.0")
        .expect("tag push succeeds");

    assert_eq!(git(&fixture.repo, &["rev-parse", "HEAD"]), commit);
    assert_eq!(
        git(&fixture.remote, &["rev-parse", "main"]),
        commit,
        "the bare remote has the release commit"
    );
    let tag_type = git(&fixture.remote, &["cat-file", "-t", "v0.2.0"]);
    assert_eq!(tag_type, "tag", "the pushed tag is annotated");
    // An annotated tag resolves to its tag object; peel it to reach the commit.
    assert_eq!(
        git(&fixture.remote, &["rev-parse", "v0.2.0^{commit}"]),
        commit
    );

    assert_eq!(
        observed.operations(),
        &[
            GitOp::Stage(vec!["Cargo.toml".to_owned()]),
            GitOp::Commit("chore(release): v0.2.0".to_owned()),
            GitOp::Tag("v0.2.0".to_owned()),
            GitOp::Push("refs/heads/main".to_owned()),
            GitOp::Push("refs/tags/v0.2.0".to_owned()),
        ]
    );
}

/// Staging nothing is refused, so a cut can never create an empty release commit.
#[rstest]
fn staging_an_empty_set_is_refused(fixture: Fixture) {
    let mut observed = LocalGit::open(&fixture.repo).expect("open");

    let error = observed.stage(&[]).expect_err("nothing to stage");

    assert_eq!(error.operation(), "stage");
    assert!(observed.operations().is_empty());
}
