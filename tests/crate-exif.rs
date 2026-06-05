#[test]
fn exif_user_comment_gthumb() {
    let data = std::fs::read("test-images/exif/gthumb/user_comment.jpg").unwrap();
    let metadata = gufo::Metadata::for_guessed(data).unwrap();
    assert_eq!(
        metadata.user_comment().as_deref(),
        Some("A somewhat longer comment")
    );
}

#[test]
fn exif_delete_entry() {
    let data = std::fs::read("test-images/exif/gthumb/user_comment.jpg").unwrap();
    let (metadata, _) = gufo::RawMetadata::for_guessed(data).unwrap();
    let data = metadata.exif[0].clone();

    let mut exif = gufo_exif::ExifOwned::for_vec(data).unwrap();
    let deleted = exif.delete(gufo_common::field::UserComment.into()).unwrap();
    assert_eq!(deleted, true);
}
