use gufo_jpeg::EXIF_IDENTIFIER_STRING;

#[test]
fn exif() {
    let data = std::fs::read("exif-xmp.jpg").unwrap();
    let jpeg = gufo_jpeg::Jpeg::new(data).unwrap();

    let exif = gufo_exif::Exif::new(jpeg.exif_data().next().unwrap().to_vec()).unwrap();

    assert_eq!(
        exif.orientation(),
        Some(gufo_common::orientation::Orientation::Id)
    );

    assert_eq!(exif.model(), Some(String::from("iPhone 6")));
}

#[test]
fn xmp() {
    let data = std::fs::read("exif-xmp.jpg").unwrap();
    let jpeg = gufo_jpeg::Jpeg::new(data).unwrap();

    let xmp = gufo_xmp::Xmp::new(jpeg.xmp_data().next().unwrap().to_vec()).unwrap();

    assert_eq!(
        xmp.get(gufo_xmp::Tag::new(
            gufo_common::xmp::Namespace::Xmp,
            "CreatorTool".into()
        )),
        Some("GIMP 2.10")
    );
}

#[test]
fn rotate() {
    let data = std::fs::read("exif-xmp.jpg").unwrap();

    let jpeg = gufo_jpeg::Jpeg::new(data).unwrap();
    let mut exif = gufo_exif::internal::ExifRaw::new(jpeg.exif_data().next().unwrap().to_vec());

    exif.decode().unwrap();
    let entry = exif.lookup_entry(gufo_common::field::Orientation).unwrap();

    let pos = jpeg.exif_segments().next().unwrap().data_pos() as usize
        + entry.value_offset_position() as usize
        + EXIF_IDENTIFIER_STRING.len();

    let mut data = jpeg.into_inner();

    let current_orientation =
        gufo_common::orientation::Orientation::try_from(data[pos] as u16).unwrap();

    let new_rotation = current_orientation.rotate() + gufo_common::orientation::Rotation::_180;

    let new_orientation =
        gufo_common::orientation::Orientation::new(current_orientation.mirror(), new_rotation);

    data[pos] = new_orientation as u8;

    let jpeg = gufo_jpeg::Jpeg::new(data).unwrap();
    let exif = gufo_exif::Exif::new(jpeg.exif_data().next().unwrap().to_vec()).unwrap();
    assert_eq!(
        exif.orientation(),
        Some(gufo_common::orientation::Orientation::Rotation180)
    );
}
