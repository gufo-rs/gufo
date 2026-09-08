use std::assert_matches;

use gufo_common::image::ImageMetadata;

#[test]
fn png_overwrite_exif() {
    let data = std::fs::read("test-images/exif/png/canon-400d-exif-eXIf.png").unwrap();

    let mut png = gufo_png::Png::new(data).unwrap();
    let exif_data = png.exif().pop().unwrap();
    let mut exif = gufo_exif::Exif::for_vec(exif_data).unwrap();
    assert!(exif.camera_owner_name().is_some());
    assert_matches!(
        exif.delete(gufo_common::field::CanonCameraOwnerName.into()),
        Ok(true)
    );

    png.set_exif(&exif.serialize().unwrap()).unwrap();
    let new_data = png.into_inner();

    // Check result
    let changed_png = gufo_png::Png::new(new_data).unwrap();
    let mut exif_data = changed_png.exif();
    let new_exif = gufo_exif::Exif::for_vec(exif_data.pop().unwrap()).unwrap();

    assert_matches!(exif_data.pop(), None);
    assert_matches!(new_exif.camera_owner_name(), None);
}

#[test]
fn png_overwrite_exif_key_value() {
    // Take Exif chunk from other image
    let data = std::fs::read("test-images/exif/png/canon-400d-exif-eXIf.png").unwrap();

    let source_png = gufo_png::Png::new(data).unwrap();
    let exif_data = source_png.exif().pop().unwrap();
    let mut exif = gufo_exif::Exif::for_vec(exif_data).unwrap();
    assert!(exif.camera_owner_name().is_some());
    assert_matches!(
        exif.delete(gufo_common::field::CanonCameraOwnerName.into()),
        Ok(true)
    );

    //
    let data = std::fs::read("test-images/exif/png/canon-400d-exif-key-value.png").unwrap();
    let mut png = gufo_png::Png::new(data).unwrap();
    assert_eq!(
        png.key_value().get("exif:Software").map(|x| x.as_str()),
        Some("GIMP 2.10.38")
    );
    png.set_exif(&exif.serialize().unwrap()).unwrap();
    let new_data = png.into_inner();

    // Check result
    let changed_png = gufo_png::Png::new(new_data).unwrap();
    let mut exif_data = changed_png.exif();
    let new_exif = gufo_exif::Exif::for_vec(exif_data.pop().unwrap()).unwrap();

    // Check removed legacy key-value exif entries
    assert_matches!(changed_png.key_value().get("exif:Software"), None);

    // Check exif data available
    assert_matches!(exif_data.pop(), None);
    assert_matches!(
        new_exif.software().as_ref().map(|x| x.as_str()),
        Some("GIMP 2.10.38")
    );
}
