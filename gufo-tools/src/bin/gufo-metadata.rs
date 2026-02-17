use std::fmt::Debug;
use tracing_subscriber::prelude::*;

fn main() {
    let path = std::env::args().nth(1).unwrap();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::builder().from_env_lossy())
        .with(tracing_subscriber::fmt::Layer::default().compact())
        .init();

    let file_data = std::fs::read(path).unwrap();

    let m = gufo::Metadata::for_guessed(file_data).unwrap();

    print_option("Creator", m.creator());
    print_option("Physical Dimensions", m.phyiscal_dimensions());
}

fn print_option(title: &str, value: Option<impl Debug>) {
    let v = match value {
        Some(v) => format!("{:?}", v),
        None => String::from("–"),
    };

    println!("{title}:\t{v}");
}
