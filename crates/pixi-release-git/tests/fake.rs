use pixi_release_git::{FakeGit, GitProtocol};

#[test]
fn fake_git_exposes_stateful_release_preconditions() {
    let mut git = FakeGit::new("0123456789abcdef", true);

    assert_eq!(git.head_sha(), "0123456789abcdef");
    assert!(git.is_clean());

    git.set_clean(false);
    assert!(!git.is_clean());
}
