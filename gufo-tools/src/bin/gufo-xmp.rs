use std::collections::BTreeSet;

use gufo_tools::*;
use gufo_xmp::Value;
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

    println!("{}", String::from_utf8_lossy(&xmp_data));
    print(xmp_data);
}

fn print(xmp_data: Vec<u8>) {
    let x = gufo_xmp::Xmp::new(xmp_data).unwrap();

    println!("{}", output(&x));

    show("Camera Owner Name", x.camera_owner_name());
    show("Creator", x.creator());
    show("DateTime Original", x.date_time_original());
    show(
        "Digital Zoom Ratio",
        x.digital_zoom_ratio()
            .map(|x| format!("{}\u{00D7}", x.as_f32())),
    );
    show("Exposure Time", x.exposure_time().map(|x| x.display()));
    show("F-Number", x.f_number());
    show("Focal Length", x.focal_length().map(|x| x.as_f32()));
    //show("GPS Location", x.gps_location().map(|x| x.iso_6709()));
    show("ISO Speed Rating", x.iso_speed_rating());
    show("Lens Make", x.lens_make());
    show("Lens Model", x.lens_model());
    /* show(
        "Lens Sepcification",
        x.lens_specification().map(|x| x.display()),
    ); */
    show("Make", x.make());
    show("Model", x.model());
    show_("Orientation", x.orientation());
    show("Rights", x.rights());
    show("Rights Web Statement", x.rights_web_statement());
    show("Software", x.software());
    show("User Comment", x.user_comment());
}

pub fn output(xmp: &gufo_xmp::Xmp) -> String {
    let mut s = String::new();

    let namespaces = BTreeSet::from_iter(xmp.entries().iter().map(|x| x.0.namespace().to_url()));

    for namespace in namespaces {
        s.push_str(namespace);
        s.push('\n');
        for (tag, value) in xmp.entries() {
            if tag.namespace().to_url() == namespace {
                let v = match value {
                    Value::Generic(s) => s.to_string(),
                    Value::Bag(vec) => vec.join(", "),
                };
                s.push_str(&format!("{:>30}: {v}\n", tag.name()));
            }
        }
        s.push('\n');
    }

    s
}
