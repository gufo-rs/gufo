use gufo_png::{ChunkType, Png};

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("First agument must be a path.");
    let data = std::fs::read(path).unwrap();
    let png = Png::new(&data).unwrap();

    for chunk in png.chunks() {
        match chunk.chunk_type() {
            ChunkType::Unknown(unknown) => println!(
                "Unknown({})",
                String::from_utf8_lossy(&u32::to_be_bytes(unknown))
            ),
            chunk_type => println!("{chunk_type:?}"),
        }

        if chunk.chunk_type() == ChunkType::tEXt {
            println!(
                " {}",
                String::from_utf8_lossy(chunk.data()).replace('\0', " ␀ ")
            );
        }
    }
}
