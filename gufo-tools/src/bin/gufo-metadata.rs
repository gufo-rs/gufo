use std::fmt::{Debug, Display};

use tracing_subscriber::prelude::*;

fn main() {
    let path = std::env::args().nth(1).unwrap();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::builder().from_env_lossy())
        .with(tracing_subscriber::fmt::Layer::default().compact())
        .init();

    let image_data = std::fs::read(path).unwrap();

    let metadata = gufo::Metadata::for_guessed(image_data).unwrap();

    print(metadata);
}

fn print(metadata: gufo::Metadata) {
    show_("Camera Owner Name", metadata.camera_owner_name());
    show("DateTime Original", metadata.date_time_original());
    show(
        "Exposure Time",
        metadata.exposure_time().map(|x| x.display()),
    );
    show("F-Number", metadata.f_number());
    show("Focal Length", metadata.focal_length());
    show(
        "GPS Location",
        metadata.gps_location().map(|x| x.iso_6709()),
    );
    show("ISO Speed Rating", metadata.iso_speed_rating());
    show("Make", metadata.make());
    show("Model", metadata.model());
    show_("Orientation", metadata.orientation());
    show("Software", metadata.software());
    show("User Comment", metadata.user_comment());
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
