use gufo_exif::Exif;
mod utils;
use utils::*;

#[test]
fn canon() {
    let raw = get_exif("tests/test-images/images/exif-makernote/canon-eos400d.jpg");
    let exif = Exif::new(raw).unwrap();

    eprintln!("{}", exif.debug_dump());

    assert_eq!(exif.make().unwrap().as_str(), "Canon");
    assert_eq!(exif.model().unwrap().as_str(), "Canon EOS 400D DIGITAL");
    assert_eq!(exif.iso_speed_rating().unwrap(), 200);
    assert_eq!(exif.f_number().unwrap(), 5.6);
    assert_eq!(exif.focal_length().unwrap(), 53.);
    assert_eq!(exif.exposure_time().unwrap(), (1, 60));
    assert_eq!(exif.date_time_original().unwrap(), "2007-10-19T19:57:06");
}

#[test]
fn apple() {
    let raw = get_exif("tests/test-images/images/exif-makernote/apple-iphone6.jpg");
    let exif = Exif::new(raw).unwrap();

    assert_eq!(exif.make().unwrap().as_str(), "Apple");
    assert_eq!(exif.model().unwrap().as_str(), "iPhone 6");
    assert_eq!(exif.iso_speed_rating().unwrap(), 32);
    assert_eq!(exif.f_number().unwrap(), 2.2);
    assert_eq!(exif.focal_length().unwrap(), 4.15);
    assert_eq!(exif.exposure_time().unwrap(), (1, 682));
    assert_eq!(exif.date_time_original().unwrap(), "2021-02-13T14:12:20");
}
