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

    p("Model", metadata.model());
    p("Creator", metadata.creator());
}

pub fn p(label: &str, s: Option<String>) {
    if let Some(s) = s {
        println!("{label}: {s}");
    } else {
        println!("{label}: –");
    }
}
