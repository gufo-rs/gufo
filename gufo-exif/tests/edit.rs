mod utils;
use gufo_common::exif::Field;
use gufo_common::field;
use utils::*;

#[test]
fn insert_at_exif() {
    let mut exif = get_decoded_exif("tests/test-images/exif/with-thumbnail.jpg");
    let tagifd = TagIfd::new(field::ImageWidth::TAG, Ifd::Exif);
    let check_tagifd = TagIfd::new(field::ImageWidth::TAG, Ifd::Thumbnail);

    eprintln!("{}", exif.debug_dump());

    let value = exif.lookup_short(tagifd).unwrap();
    assert_eq!(value, None);

    let value = exif.lookup_binary(check_tagifd).unwrap().unwrap();
    assert_eq!(value, vec![0, 1, 0, 0]);

    for _ in 0..3 {
        exif.insert_entry(
            tagifd,
            EntryRef {
                count: 1,
                data_type: Type::Short,
                position: 0,
                value_offset: ValueOffset::Value(6),
            },
        )
        .unwrap();

        let value: Option<u16> = exif.lookup_short(tagifd).unwrap();
        assert_eq!(value, Some(6));

        let value = exif.lookup_binary(check_tagifd).unwrap().unwrap();
        assert_eq!(value, vec![0, 1, 0, 0]);
    }

    exif.decode().unwrap();

    let value: Option<u16> = exif.lookup_short(tagifd).unwrap();
    assert_eq!(value, Some(6));

    let value = exif.lookup_binary(check_tagifd).unwrap().unwrap();
    assert_eq!(value, vec![0, 1, 0, 0]);
}

#[test]
fn insert_at_thumbnail() {
    let mut exif = get_decoded_exif("tests/test-images/exif/with-thumbnail.jpg");
    let tagifd = field::ThumbnailOrientation;

    let value = exif.lookup_short(tagifd).unwrap();
    assert_eq!(value, None);

    exif.insert_entry(
        tagifd,
        EntryRef {
            count: 1,
            data_type: Type::Short,
            position: 0,
            value_offset: ValueOffset::Value(5),
        },
    )
    .unwrap();

    let value = exif.lookup_short(tagifd).unwrap();
    assert_eq!(value, Some(5));
}
