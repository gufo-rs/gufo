#[test]
fn user_comment() {
    let data = std::fs::read("test-images/exif/gthumb/user_comment.jpg").unwrap();
    let metadata = dbg!(gufo::Metadata::for_guessed(data).unwrap());
    assert_eq!(
        metadata.user_comment().as_deref(),
        Some("A somewhat longer comment")
    );
}
