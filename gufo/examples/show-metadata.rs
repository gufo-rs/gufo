pub fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("First agument must be a path.");
    let data = std::fs::read(path).unwrap();

    let png = gufo_png::Png::new(data).unwrap();
    let metadata = gufo::Metadata::for_png(&png);

    println!("Model: {:?}", metadata.model());
}
