use std::usize;

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

        match chunk.chunk_type() {
            ChunkType::tEXt => {
                let (keyword, text) = chunk.text().unwrap();
                println!(
                    " {}:\n{}\n",
                    String::from_utf8_lossy(keyword),
                    String::from_utf8_lossy(text)
                );
            }

            ChunkType::iTXt => {
                println!(" {}\n", String::from_utf8_lossy(chunk.keyword().unwrap()));
            }

            ChunkType::zTXt => {
                let (keyword, data) = chunk.ztxt(usize::MAX).unwrap();
                println!(
                    " {}:\n{}\n",
                    String::from_utf8_lossy(keyword),
                    String::from_utf8_lossy(&data[..15])
                );

                dbg!(String::from_utf8_lossy(
                    &chunk.legacy_exif(usize::MAX).unwrap()[0..10]
                ));
            }
            ChunkType::eXIf => {
                println!(" {}\n", String::from_utf8_lossy(&chunk.data()[..2]));
            }
            _ => (),
        }
    }
}
