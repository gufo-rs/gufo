use std::fmt::Display;

pub fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("First agument must be a path.");
    let data = std::fs::read(path).unwrap();

    let metadata = match gufo::Metadata::for_guessed(data) {
        Ok(metadata) => metadata,
        Err(err) => {
            dbg!(&err);
            eprintln!("\n{}", err);
            return;
        }
    };

    p("Created", metadata.date_time_original());
    p("Model", metadata.model());
    p("Make", metadata.make());
    p("F-Number", metadata.f_number().map(|x| format!("f/{x}")));
    p(
        "Exposure Time",
        metadata.exposure_time().map(|(x, y)| format!("{x}/{y} s")),
    );
    p("ISO", metadata.iso_speed_rating());
    p(
        "Focal length",
        metadata.focal_length().map(|x| format!("{x} mm")),
    );

    p("Creator", metadata.creator());
}

pub fn p(label: &str, s: Option<impl Display>) {
    if let Some(s) = s {
        println!("{label}: {s}");
    } else {
        println!("{label}: –");
    }
}
