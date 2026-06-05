use std::fmt::{Debug, Display};

use gufo_exif::{Exif, Storage};
use tracing_subscriber::prelude::*;

fn main() {
    let path = std::env::args().nth(1).unwrap();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::builder().from_env_lossy())
        .with(tracing_subscriber::fmt::Layer::default().compact())
        .init();

    let image_data = std::fs::read(path).unwrap();

    let (mut raw_metadata, _) = gufo::RawMetadata::for_guessed(image_data).unwrap();

    let exif_data = raw_metadata.exif.pop().unwrap();

    print(exif_data);
}

fn print(exif_data: Vec<u8>) {
    let mut exif = gufo_exif::ExifOwned::for_vec(exif_data).unwrap();

    eprintln!("{}", output(&mut exif));

    show_("Camera Owner Name", exif.camera_owner_name());
    show("DateTime Original", exif.date_time_original());
    show("Exposure Time", exif.exposure_time().map(|x| x.display()));
    show("F-Number", exif.f_number());
    show("Focal Length", exif.focal_length());
    show("GPS Location", exif.gps_location().map(|x| x.iso_6709()));
    show("ISO Speed Rating", exif.iso_speed_rating());
    show("Make", exif.make());
    show("Model", exif.model());
    show_("Orientation", exif.orientation());
    show("Software", exif.software());
    show("User Comment", exif.user_comment());
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

pub fn output<'a, S: Storage<'a>>(exif: &mut Exif<'a, S>) -> String {
    exif.document(|document| {
        let mut s = String::new();

        let entries = document.entries().unwrap();
        for (ifd, (pos, _)) in document.ifds().iter_mut() {
            s.push_str(&format!("\n{ifd:?} ({pos})\n----------\n"));

            if let Some(entries) = entries.get(ifd) {
                for entry in entries.values() {
                    let tag_name = gufo_common::exif::lookup_tag_name(entry.tag_ifd);
                    let row = format!(
                        "{tag:25} {count:2}×{type_:10}",
                        tag = tag_name.unwrap_or(&format!("Unknown({})", entry.tag_ifd.tag.0)),
                        type_ = format!("{:?}", entry.type_),
                        count = entry.count,
                    );

                    s.push_str(&row);

                    let data = match &entry.data {
                        Ok(data) => data.display(),
                        Err(err) => format!("Error: {err}"),
                    };

                    if data.len() > 40 {
                        s.push_str(&format!("\n  {data}\n"));
                    } else {
                        s.push_str(&format!(" {data}\n"));
                    }
                }
            }
        }

        s.push_str("\nDatablocks\n");
        for (start, data) in document.data_blocks() {
            s.push_str(&format!(
                "{start}-{} ({})\n",
                start + data.len(),
                data.len()
            ));
        }

        s
    })
}
