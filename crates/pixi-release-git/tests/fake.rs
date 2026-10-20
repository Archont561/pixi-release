use pixi_release_git::{FakeGit, GitProtocol};
use rstest::{fixture, rstest};

#[fixture]
fn clean_git() -> FakeGit {
    FakeGit::new("0123456789abcdef", true)
}

#[rstest]
fn fake_git_exposes_stateful_release_preconditions(mut clean_git: FakeGit) {
    assert_eq!(clean_git.head_sha(), "0123456789abcdef");
    assert!(clean_git.is_clean());

    clean_git.set_clean(false);
    assert!(!clean_git.is_clean());
}
