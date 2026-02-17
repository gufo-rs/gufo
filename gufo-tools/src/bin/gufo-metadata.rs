use gufo_tools::*;
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

fn print(x: gufo::Metadata) {
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
    show("Rights", x.rights());
    show("Rights Web Statement", x.rights_web_statement());
    show("Software", x.software());
    show("User Comment", x.user_comment());

    let mut print = Print::new(vec![Alignment::Right, Alignment::Left]);

    print.option_dbg("Creator", x.creator());
    print.option_dsp(
        "Physical Dimensions",
        x.pixel_density().map(|x| x.display()),
    );

    print.print();
}

struct Print {
    rows: Vec<Vec<String>>,
    alignment: Vec<Alignment>,
}

enum Alignment {
    Left,
    Right,
}

impl Print {
    fn new(alignment: Vec<Alignment>) -> Self {
        Self {
            rows: Default::default(),
            alignment,
        }
    }

    fn option_dbg(&mut self, title: &str, value: Option<impl Debug>) {
        let v = match value {
            Some(v) => format!("{:?}", v),
            None => String::from("–"),
        };

        self.rows.push(vec![format!("{title}: "), v]);
    }

    fn option_dsp(&mut self, title: &str, value: Option<impl Display>) {
        let v = match value {
            Some(v) => v.to_string(),
            None => String::from("–"),
        };

        self.rows.push(vec![format!("{title}: "), v]);
    }

    fn print(&self) {
        let mut col_width = Vec::new();
        for row in &self.rows {
            for (n, col) in row.iter().enumerate() {
                let w = col.chars().count();
                if let Some(current_width) = col_width.get_mut(n) {
                    if w > *current_width {
                        *current_width = w;
                    }
                } else {
                    col_width.push(w);
                }
            }
        }

        for row in &self.rows {
            for (n, col) in row.iter().enumerate() {
                let width_diff = col_width[n] - col.chars().count();
                match self.alignment[n] {
                    Alignment::Left => print!("{col}{spaces}", spaces = " ".repeat(width_diff)),
                    Alignment::Right => print!("{spaces}{col}", spaces = " ".repeat(width_diff)),
                }
            }
            print!("\n");
        }
    }
}
