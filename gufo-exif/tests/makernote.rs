mod utils;
use gufo_common::exif::Field;
use gufo_common::field;
use utils::*;

#[test]
fn lookup() {
    test_tag_binary(
        "tests/test-images/images/exif-makernote/canon-eos400d.jpg",
        TagIfd::new(Tag(6), Ifd::MakerNote),
        b"Canon EOS 400D DIGITAL\0",
    );
}

#[test]
fn insert_moving_makernote() {
    let mut decoder = get_decoded_exif("tests/test-images/images/exif-makernote/canon-eos400d.jpg");

    let result = decoder
        .lookup_binary(field::DateTimeOriginal)
        .unwrap()
        .unwrap();
    assert_eq!(result.as_slice(), b"2007:10:19 19:57:06\0");

    let result = decoder
        .lookup_binary(TagIfd::new(Tag(6), Ifd::MakerNote))
        .unwrap()
        .unwrap();
    assert_eq!(result.as_slice(), b"Canon EOS 400D DIGITAL\0");

    decoder
        .insert_entry(
            field::Orientation,
            EntryRef {
                position: 0,
                data_type: Type::Short,
                count: 1,
                value_offset: ValueOffset::Value(5),
            },
        )
        .unwrap();

    let result = decoder
        .lookup_binary(field::DateTimeOriginal)
        .unwrap()
        .unwrap();
    assert_eq!(result.as_slice(), b"2007:10:19 19:57:06\0");

    let result = decoder
        .lookup_binary(TagIfd::new(Tag(6), Ifd::MakerNote))
        .unwrap()
        .unwrap();
    assert_eq!(result.as_slice(), b"Canon EOS 400D DIGITAL\0");
}

fn data() -> Vec<u8> {
    let mut data = Vec::new();

    // Litte endian
    data.extend_from_slice(b"II");
    // Magic bits
    data.extend_from_slice(&[42, 0]);
    // Offset
    data.extend_from_slice(&8_u32.to_le_bytes());
    // Number entries
    data.extend_from_slice(&3_u16.to_le_bytes());

    // Tag orientation
    data.extend_from_slice(&field::Orientation::TAG.0.to_le_bytes());
    // Data type
    data.extend_from_slice(&Type::Ascii.u16().to_le_bytes());
    // Count
    data.extend_from_slice(&7_u32.to_le_bytes());
    // Offset
    data.extend_from_slice(&60_u32.to_le_bytes());

    // Tag orientation
    data.extend_from_slice(&field::XResolution::TAG.0.to_le_bytes());
    // Data type
    data.extend_from_slice(&Type::Ascii.u16().to_le_bytes());
    // Count
    data.extend_from_slice(&5_u32.to_le_bytes());
    // Offset
    data.extend_from_slice(&70_u32.to_le_bytes());

    // Tag
    data.extend_from_slice(&Tag::MAKER_NOTE.0.to_le_bytes());
    // Data type
    data.extend_from_slice(&Type::Undefined.u16().to_le_bytes());
    // Count
    data.extend_from_slice(&10_u32.to_le_bytes());
    // Value
    data.extend_from_slice(&80_u32.to_le_bytes());

    // Next offset
    data.extend_from_slice(&[0; 4]);

    data.extend_from_slice(&[1; 10]);
    data.extend_from_slice(b"abcdefg");

    data.extend_from_slice(&[2; 3]);
    data.extend_from_slice(b"hijkl");

    data.extend_from_slice(&[3; 5]);
    data.extend_from_slice(b"ABCDEFGHIJ");

    data
}

#[test]
fn abc() {
    let mut exif = ExifRaw::new(data());
    exif.decode().unwrap();

    let tag1 = field::Orientation;
    let tag2 = field::XResolution;
    let tag3 = TagIfd::new(Tag::MAKER_NOTE, Ifd::Primary);

    let value = exif.lookup_binary(tag1).unwrap().unwrap();
    assert_eq!(value.as_slice(), b"abcdefg");

    let value = exif.lookup_binary(tag2).unwrap().unwrap();
    assert_eq!(value.as_slice(), b"hijkl");

    let value = exif.lookup_binary(tag3).unwrap().unwrap();
    assert_eq!(value.as_slice(), b"ABCDEFGHIJ");

    assert_eq!(exif.last_data_end_before(80).unwrap(), 75);

    exif.freeup_space_before(80, 5, 90).unwrap();

    let value = exif.lookup_binary(tag1).unwrap().unwrap();
    assert_eq!(value.as_slice(), b"abcdefg");

    let value = exif.lookup_binary(tag2).unwrap().unwrap();
    assert_eq!(value.as_slice(), b"hijkl");

    let value = exif.lookup_binary(tag3).unwrap().unwrap();
    assert_eq!(value.as_slice(), b"ABCDEFGHIJ");
}
