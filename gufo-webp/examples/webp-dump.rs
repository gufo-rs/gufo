fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("First agument must be a path.");
    let data = std::fs::read(path).unwrap();
    let webp = gufo_webp::WebP::new(data).unwrap();

    for chunk in webp.chunks() {
        match chunk.four_cc() {
            gufo_webp::FourCC::Unknown(unknown) => println!(
                "Unknown({})",
                String::from_utf8_lossy(&u32::to_le_bytes(unknown))
            ),
            chunk_type => println!("{chunk_type:?}"),
        }
    }
}
