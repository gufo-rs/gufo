use std::assert_matches;

use gufo_common::image::ImageMetadata;

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

#[test]
fn exif_delete_thumbnail() {
    let data = std::fs::read("test-images/exif/jpeg/exif-thumbnail.jpg").unwrap();
    let jpeg = gufo_jpeg::Jpeg::new(data).unwrap();
    let exif_data = jpeg.exif().pop().unwrap();

    let mut exif = gufo_exif::Exif::for_vec(exif_data).unwrap();

    let thumbnail_data = exif.thumbnail().unwrap();
    let thumbnail_jpeg = gufo_jpeg::Jpeg::new(thumbnail_data).unwrap();
    let sof = thumbnail_jpeg.sof().unwrap();

    // Check we really got the thumbnail
    assert_eq!(sof.x, 5);
    assert_eq!(sof.y, 5);

    assert!(exif.delete_thumbnail().unwrap());

    assert_matches!(exif.thumbnail(), None);
}
