use gufo::common::image::ImageMetadata;
use tracing_subscriber::prelude::*;

fn main() {
    let path = std::env::args().nth(1).unwrap();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::builder().from_env_lossy())
        .with(tracing_subscriber::fmt::Layer::default().compact())
        .init();

    let image_data = std::fs::read(path).unwrap();
    let image = gufo::Image::new(image_data).unwrap();

    match image {
        gufo::Image::Png(png) => show_png(png),
        gufo::Image::Jpeg(jpeg) => show_jpeg(jpeg),
        unknown => panic!("Unknown file type: {unknown:?}"),
    }
}

fn show_png(png: gufo::png::Png) {
    fn show_repeats(n: &mut u32, chunk_type: &gufo::png::ChunkType) {
        if *n > 1 {
            println!(" - {chunk_type:?} ({n}x)");
            *n = 1;
        } else {
            println!(" - {chunk_type:?}");
        }
    }

    println!("PNG Chunks:");
    let mut n_repeats = 1;
    let mut last_type = gufo::png::ChunkType::IEND;
    for chunk in png.chunks() {
        if let Ok((keyword, value)) = chunk.textual(200000) {
            let chunk_type = chunk.chunk_type();
            let keyword = String::from_utf8_lossy(keyword);
            let value = String::from_utf8_lossy(&value);
            println!(" - {chunk_type:?}: {keyword}\n   {value}");

            last_type = chunk_type;
        } else if chunk.chunk_type() != last_type {
            show_repeats(&mut n_repeats, &last_type);
            last_type = chunk.chunk_type();
        } else {
            n_repeats += 1;
        }
    }
    show_repeats(&mut n_repeats, &last_type);

    println!("Physical Dimensions: {:?}", png.pixel_density());
}

fn show_jpeg(jpeg: gufo::jpeg::Jpeg) {
    println!("JPEG Segments:");
    for segment in jpeg.segments() {
        println!(
            " - {:?} ({} bytes) {}",
            segment.marker(),
            segment.data().len(),
            segment.marker().and_then(|x| x.info()).unwrap_or_default()
        );
    }
    println!("DQT:");
    for (i, _) in jpeg.dqts().unwrap() {
        println!(" - Tq: {i}");
    }
    println!("Color Model: {:?}", jpeg.color_model().unwrap());

    println!("JFIF: {:#?}", jpeg.jfif().unwrap());
    println!("SOF: {:#?}", jpeg.sof().unwrap());
}
