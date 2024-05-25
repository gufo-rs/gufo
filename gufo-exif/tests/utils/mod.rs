#![allow(dead_code)]

pub use gufo_exif::internal::*;
pub use gufo_jpeg::Jpeg;

pub fn get_exif(path: impl AsRef<std::path::Path>) -> Vec<u8> {
    let image_data = std::fs::read(path.as_ref()).unwrap();
    let image = Jpeg::new(&image_data);

    let mut iter = image.exif_data();
    iter.next().unwrap().to_vec()
}

pub fn get_decoded_exif(path: impl AsRef<std::path::Path>) -> ExifRaw {
    let raw = get_exif(path);
    let mut exif = ExifRaw::new(raw);
    exif.decode().unwrap();
    exif.makernote_register().unwrap();
    exif
}

pub fn test_tag_binary(path: impl AsRef<std::path::Path>, tagifd: TagIfd, expected: &[u8]) {
    let expected = expected.to_vec();
    let mut exif = get_decoded_exif(path);
    let data = exif.lookup_binary(tagifd).unwrap();

    assert_eq!(data, Some(expected));
}
