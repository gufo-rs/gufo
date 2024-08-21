use std::ffi::OsStr;

use gufo_exif::internal::ExifRaw;
use gufo_jpeg::Jpeg;

#[test]
fn all_jpgs() {
    let dirs = ["tests/test-images/exif"];

    for dir in dirs {
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            if entry.path().extension() == Some(OsStr::new("jpg")) {
                load_file(&entry.path());
            }
        }
    }
}

fn load_file(path: &std::path::Path) {
    let image_data = std::fs::read(path).unwrap();

    let image = Jpeg::new(image_data).unwrap();

    let mut iter = image.exif_data();
    if let Some(exif_raw) = iter.next() {
        let mut decoder = ExifRaw::new(exif_raw.to_vec());
        decoder.decode().unwrap();
    } else {
        eprintln!("No exif data found {path:?}");
    }
}
