use std::{
    collections::BTreeSet,
    fmt::{Debug, Display},
};

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

    show_("Camera Owner Name", xmp.camera_owner_name());
    show("DateTime Original", xmp.date_time_original());
    show("Exposure Time", xmp.exposure_time().map(|x| x.display()));
    show("F-Number", xmp.f_number());
    show("Focal Length", xmp.focal_length());
    //show("GPS Location", exif.gps_location().map(|x| x.iso_6709()));
    show("ISO Speed Rating", xmp.iso_speed_rating());
    show("Make", xmp.make());
    show("Model", xmp.model());
    show_("Orientation", xmp.orientation());
    show("Software", xmp.software());
    show("User Comment", xmp.user_comment());
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
    let mut s = String::new();

    let namespaces = BTreeSet::from_iter(xmp.entries().iter().map(|x| x.0.namespace().to_url()));

    for namespace in namespaces {
        s.push_str(namespace);
        s.push('\n');
        for (tag, value) in xmp.entries() {
            if tag.namespace().to_url() == namespace {
                s.push_str(&format!("{:>30}: {value}\n", tag.name()));
            }
        }
        s.push('\n');
    }

    s
}
