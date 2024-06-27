use gufo_common::exif::Field;
use gufo_common::{field, orientation};
use gufo_exif::error::Error;
use gufo_exif::internal::*;
use gufo_exif::Exif;

fn data() -> Vec<u8> {
    let mut data = Vec::new();

    // Litte endian
    data.extend_from_slice(b"II");
    // Magic bits
    data.extend_from_slice(&[42, 0]);
    // Offset
    data.extend_from_slice(&8_u32.to_le_bytes());
    // Number entries
    data.extend_from_slice(&1_u16.to_le_bytes());

    // Tag orientation
    data.extend_from_slice(&0x112_u16.to_le_bytes());
    // Data type
    data.extend_from_slice(&3_u16.to_le_bytes());
    // Count
    data.extend_from_slice(&1_u32.to_le_bytes());
    // Value
    data.extend_from_slice(&7_u32.to_le_bytes());

    // Next offset
    data.extend_from_slice(&[0, 0, 0, 0]);

    data
}

#[test]
fn basic_low_level() {
    let mut decoder = ExifRaw::new(data());
    decoder.decode().unwrap();

    let tagifd = field::Orientation;

    let orientation = decoder.lookup_data(tagifd);
    assert_eq!(
        orientation.unwrap(),
        Some((Type::Short, 7_u32.to_le_bytes().to_vec()))
    );

    decoder.set_existing(tagifd, 5).unwrap();

    let missing_ref = TagIfd::new(field::Orientation::TAG, Ifd::Thumbnail);
    assert!(decoder.lookup_data(missing_ref).err().is_none());

    let err = decoder.set_existing(missing_ref, 1).err();
    assert!(matches!(err, Some(Error::TagNotFound(_))));
}

#[test]
fn basic_high_level() {
    let exif = Exif::new(data()).unwrap();
    assert_eq!(
        exif.orientation(),
        orientation::Orientation::MirroredRotation270
    );
}
