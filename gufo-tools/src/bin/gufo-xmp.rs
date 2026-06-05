use std::fmt::{Debug, Display};

use tracing_subscriber::prelude::*;

fn main() {
    let path = std::env::args().nth(1).unwrap();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::builder().from_env_lossy())
        .with(tracing_subscriber::fmt::Layer::default().compact())
        .init();

    let image_data = std::fs::read(path).unwrap();

    let (mut raw_metadata, _) = gufo::RawMetadata::for_guessed(image_data).unwrap();

    let xmp_data = raw_metadata.xmp.pop().unwrap();

    print(xmp_data);
}

fn print(xmp_data: Vec<u8>) {
    let xmp = gufo_xmp::Xmp::new(xmp_data).unwrap();

    eprintln!("{}", output(&xmp));

    //show_("Camera Owner Name", exif.camera_owner_name());
    //show("DateTime Original", exif.date_time_original());
    show("Exposure Time", xmp.exposure_time().map(|x| x.display()));
    show("F-Number", xmp.f_number());
    show("Focal Length", xmp.focal_length());
    //show("GPS Location", exif.gps_location().map(|x| x.iso_6709()));
    show("ISO Speed Rating", xmp.iso_speed_rating());
    show("Make", xmp.make());
    show("Model", xmp.model());
    //show_("Orientation", exif.orientation());
    //show("Software", exif.software());
    //show("User Comment", exif.user_comment());
}

fn show<T: Display>(name: &str, x: Option<T>) {
    let s = match x {
        Some(x) => x.to_string(),
        None => String::from("–"),
    };

    println!("{:>20} {s}", format!("{name}:"));
}

fn show_<T: Debug>(name: &str, x: Option<T>) {
    let s = match x {
        Some(x) => format!("{x:?}"),
        None => String::from("–"),
    };

    println!("{:>20} {s}", format!("{name}:"));
}

pub fn output(xmp: &gufo_xmp::Xmp) -> String {
    String::new()
}
