use gufo_exif::internal::ExifRaw;
use gufo_jpeg::Jpeg;
use tracing_subscriber::prelude::*;

fn main() {
    let path = std::env::args().nth(1).unwrap();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::builder().from_env_lossy())
        .with(tracing_subscriber::fmt::Layer::default().compact())
        .init();

    let image_data = std::fs::read(path).unwrap();
    let image = Jpeg::new(image_data).unwrap();
    let exif_raw = image.exif_data().next().unwrap();

    let mut decoder = ExifRaw::new(exif_raw.to_vec());
    decoder.decode().unwrap();
    decoder.makernote_register().unwrap();

    println!("{}", decoder.debug_dump());
}
