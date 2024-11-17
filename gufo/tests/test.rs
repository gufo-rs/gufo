pub use gufo_common::orientation::Orientation;

#[test]
pub fn canon_eos400d() {
    let reference = Reference {
        camera_owner: Some("Sophie Herold"),
        creator: None,
        date_time_original: Some("2012-09-16 14:08:04"),
        exposure_time: Some((1, 500)),
        f_number: Some(13.),
        focal_length: Some(17.),
        gps_location: None,
        iso_speed_rating: Some(320),
        make: Some("Canon"),
        model: Some("Canon EOS 400D DIGITAL"),
        orientation: Some(Orientation::Id),
        software: Some("GIMP 2.10.38"),
        user_comment: None,
    };

    let data = std::fs::read("../test-images/exif/jpeg/canon-400d.jpg").unwrap();
    let metadata = gufo::Metadata::for_guessed(data).unwrap();
    check(&metadata, &reference);

    let data = std::fs::read("../test-images/exif/png/canon-400d-exif-eXIf.png").unwrap();
    let metadata = gufo::Metadata::for_guessed(data).unwrap();
    check(&metadata, &reference);
}

#[test]
pub fn canon_eos400d_xmp() {
    let mut reference: Reference<'_> = Default::default();
    reference.software = Some("GIMP 2.10");

    let data = std::fs::read("../test-images/exif/png/canon-400d-xmp-iTXt.png").unwrap();
    let metadata = gufo::Metadata::for_guessed(data).unwrap();
    check(&metadata, &reference);

    let data = std::fs::read("../test-images/exif/png/canon-400d-xmp-zTXt.png").unwrap();
    let metadata = gufo::Metadata::for_guessed(data).unwrap();
    check(&metadata, &reference);
}

#[test]
pub fn apple_iphone6() {
    let data = std::fs::read("../test-images/exif/jpeg/apple-iphone6.jpg").unwrap();
    let metadata = gufo::Metadata::for_guessed(data).unwrap();

    check(
        &metadata,
        &Reference {
            camera_owner: None,
            creator: None,
            date_time_original: Some("2020-07-07 14:08:27.890"),
            exposure_time: Some((1, 100)),
            f_number: Some(2.2),
            focal_length: Some(2.65),
            gps_location: Some("geo:52.543644,13.383522"),
            iso_speed_rating: Some(50),
            make: Some("Apple"),
            model: Some("iPhone 6"),
            orientation: Some(Orientation::Rotation270),
            software: Some("GIMP 2.10.38"),
            user_comment: None,
        },
    );
}

#[test]
pub fn nikon_d5100() {
    let data = std::fs::read("../test-images/exif/jpeg/nikon-d5100.jpg").unwrap();
    let metadata = gufo::Metadata::for_guessed(data).unwrap();

    check(
        &metadata,
        &Reference {
            camera_owner: None,
            // From XMP
            creator: Some("Kiley Barbero"),
            date_time_original: Some("2012-08-29 01:55:05.400"),
            exposure_time: Some((1, 500)),
            f_number: Some(5.6),
            focal_length: Some(280.),
            gps_location: None,
            iso_speed_rating: Some(320),
            make: Some("NIKON CORPORATION"),
            model: Some("NIKON D5100"),
            orientation: Some(Orientation::Id),
            software: Some("GIMP 2.10.38"),
            user_comment: None,
        },
    );
}

#[derive(Default)]
struct Reference<'a> {
    camera_owner: Option<&'a str>,
    creator: Option<&'a str>,
    date_time_original: Option<&'a str>,
    exposure_time: Option<(u32, u32)>,
    f_number: Option<f32>,
    focal_length: Option<f32>,
    gps_location: Option<&'a str>,
    iso_speed_rating: Option<u16>,
    make: Option<&'a str>,
    model: Option<&'a str>,
    orientation: Option<Orientation>,
    software: Option<&'a str>,
    user_comment: Option<&'a str>,
}

fn check(metadata: &gufo::Metadata, reference: &Reference) {
    assert_eq!(
        metadata.camera_owner(),
        reference.camera_owner.map(ToString::to_string)
    );
    assert_eq!(
        metadata.creator(),
        reference.creator.map(ToString::to_string)
    );
    assert_eq!(
        metadata.date_time_original().map(|x| x.to_string()),
        reference.date_time_original.map(ToString::to_string)
    );
    assert_eq!(metadata.exposure_time(), reference.exposure_time);
    assert_eq!(metadata.f_number(), reference.f_number);
    assert_eq!(metadata.focal_length(), reference.focal_length);
    assert_eq!(
        metadata.gps_location().map(|x| x.geo_uri()),
        reference.gps_location.map(ToString::to_string)
    );
    assert_eq!(metadata.iso_speed_rating(), reference.iso_speed_rating);
    assert_eq!(metadata.make(), reference.make.map(ToString::to_string));
    assert_eq!(metadata.model(), reference.model.map(ToString::to_string));
    assert_eq!(metadata.orientation(), reference.orientation);
    assert_eq!(
        metadata.software(),
        reference.software.map(ToString::to_string)
    );
    assert_eq!(
        metadata.user_comment(),
        reference.user_comment.map(ToString::to_string)
    );
}
