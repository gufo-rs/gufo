use gufo_exif::{Exif, Storage};
use gufo_tools::*;
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
    let mut x = gufo_exif::ExifOwned::for_vec(exif_data).unwrap();

    println!("{}", output(&mut x));

    show("Artist", x.artist());
    show("Camera Owner Name", x.camera_owner_name());
    show("Copyright", x.copyright());
    show("DateTime Original", x.date_time_original());
    show(
        "Digital Zoom Ratio",
        x.digital_zoom_ratio()
            .map(|x| format!("{}\u{00D7}", x.as_f32())),
    );
    show("Exposure Time", x.exposure_time().map(|x| x.display()));
    show("F-Number", x.f_number().map(|x| x.as_f32()));
    show("Focal Length", x.focal_length().map(|x| x.as_f32()));
    show("GPS Location", x.gps_location().map(|x| x.iso_6709()));
    show("ISO Speed Rating", x.iso_speed_rating());
    show("Lens Make", x.lens_make());
    show("Lens Model", x.lens_model());
    show(
        "Lens Sepcification",
        x.lens_specification().map(|x| x.display()),
    );
    show("Make", x.make());
    show("Model", x.model());
    show_("Orientation", x.orientation());
    show("Software", x.software());
    show("User Comment", x.user_comment());
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
