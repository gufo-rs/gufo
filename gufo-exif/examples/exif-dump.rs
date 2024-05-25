use gufo_exif::internal::ExifRaw;
use gufo_jpeg::Jpeg;

fn main() {
    let path = std::env::args().nth(1).unwrap();

    let image_data = std::fs::read(path).unwrap();
    let image = Jpeg::new(&image_data);
    let exif_raw = image.exif_data().next().unwrap();

    let mut decoder = ExifRaw::new(exif_raw.to_vec());
    decoder.decode().unwrap();
    decoder.makernote_register().unwrap();

    println!("{}", decoder.debug_dump());
}
