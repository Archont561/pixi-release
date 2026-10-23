mod support;

use rstest::{fixture, rstest};
use support::{DummyRepo, build_dummy_repo, git_output};

#[fixture]
fn dummy_repo() -> DummyRepo {
    build_dummy_repo("release-candidate")
}

#[rstest]
fn dummy_repo_is_a_committed_clean_checkout(dummy_repo: DummyRepo) {
    assert_eq!(
        git_output(dummy_repo.path(), &["status", "--porcelain"]),
        ""
    );
    assert_eq!(
        git_output(dummy_repo.path(), &["rev-parse", "--abbrev-ref", "HEAD"]),
        "main"
    );
    assert_eq!(
        git_output(dummy_repo.path(), &["rev-parse", "HEAD"]).len(),
        40
    );
}
