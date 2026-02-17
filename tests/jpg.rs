use std::io::{Cursor, Seek};

use gufo_jpeg::{Jpeg, EXIF_IDENTIFIER_STRING};
use zune_jpeg::zune_core::colorspace;

#[test]
fn exif() {
    let data = std::fs::read("exif-xmp.jpg").unwrap();
    let jpeg = gufo_jpeg::Jpeg::new(data).unwrap();

    let exif = gufo_exif::Exif::new(jpeg.exif_data().next().unwrap().to_vec()).unwrap();

    assert_eq!(
        exif.orientation(),
        Some(gufo_common::orientation::Orientation::Id)
    );

    assert_eq!(exif.model(), Some(String::from("iPhone 6")));
}

#[test]
fn xmp() {
    let data = std::fs::read("exif-xmp.jpg").unwrap();
    let jpeg = gufo_jpeg::Jpeg::new(data).unwrap();

    let xmp = gufo_xmp::Xmp::new(jpeg.xmp_data().next().unwrap().to_vec()).unwrap();

    assert_eq!(
        xmp.get(gufo_xmp::Tag::new(
            gufo_common::xmp::Namespace::Xmp,
            "CreatorTool".into()
        )),
        Some("GIMP 2.10")
    );
}

#[test]
fn rotate() {
    let data = std::fs::read("exif-xmp.jpg").unwrap();

    let jpeg = gufo_jpeg::Jpeg::new(data).unwrap();
    let mut exif = gufo_exif::internal::ExifRaw::new(jpeg.exif_data().next().unwrap().to_vec());

    exif.decode().unwrap();
    let entry = exif.lookup_entry(gufo_common::field::Orientation).unwrap();

    let pos = jpeg.exif_segments().next().unwrap().data_pos() as usize
        + entry.value_offset_position() as usize
        + EXIF_IDENTIFIER_STRING.len();

    let mut data = jpeg.into_inner();

    let current_orientation =
        gufo_common::orientation::Orientation::try_from(data[pos] as u16).unwrap();

    let new_rotation = current_orientation.rotate() + gufo_common::orientation::Rotation::_180;

    let new_orientation =
        gufo_common::orientation::Orientation::new(current_orientation.mirror(), new_rotation);

    data[pos] = new_orientation as u8;

    let jpeg = gufo_jpeg::Jpeg::new(data).unwrap();
    let exif = gufo_exif::Exif::new(jpeg.exif_data().next().unwrap().to_vec()).unwrap();
    assert_eq!(
        exif.orientation(),
        Some(gufo_common::orientation::Orientation::Rotation180)
    );
}

#[test]
pub fn re_encode_ycbcr() {
    re_encode(
        std::fs::read("test-images/images/color/color.jpg").unwrap(),
        std::fs::read("test-images/images/color.png").unwrap(),
    );
}

#[test]
pub fn re_encode_ycck() {
    re_encode(
        std::fs::read("test-images/images/color/color_cmyk.jpg").unwrap(),
        std::fs::read("test-images/images/color.png").unwrap(),
    );
}

#[test]
pub fn re_encode_luma() {
    re_encode(
        std::fs::read("test-images/images/grayscale/grayscale.jpg").unwrap(),
        std::fs::read("test-images/images/grayscale.png").unwrap(),
    );
}

fn re_encode(data: Vec<u8>, reference: Vec<u8>) {
    let jpeg = Jpeg::new(data).unwrap();

    let mut out_buf = Vec::new();
    let encoder = jpeg.encoder(&mut out_buf).unwrap();
    let mut buf = Cursor::new(jpeg.into_inner());

    let decoder_options = zune_jpeg::zune_core::options::DecoderOptions::new_fast(); //.jpeg_set_out_colorspace(zune_jpeg::zune_core::colorspace::ColorSpace::YCbCr);
    let mut decoder = zune_jpeg::JpegDecoder::new_with_options(&mut buf, decoder_options);
    decoder.decode_headers().unwrap();
    let colorspace = decoder.input_colorspace().unwrap();
    drop(decoder);

    buf.seek(std::io::SeekFrom::Start(0)).unwrap();
    let decoder_options: zune_jpeg::zune_core::options::DecoderOptions =
        zune_jpeg::zune_core::options::DecoderOptions::new_fast()
            .jpeg_set_out_colorspace(colorspace);
    let mut decoder = zune_jpeg::JpegDecoder::new_with_options(&mut buf, decoder_options);
    let pixels = decoder.decode().unwrap();
    let info: zune_jpeg::ImageInfo = decoder.info().unwrap();

    let colorspace = match colorspace {
        colorspace::ColorSpace::YCbCr => jpeg_encoder::ColorType::Ycbcr,
        colorspace::ColorSpace::Luma => jpeg_encoder::ColorType::Luma,
        colorspace::ColorSpace::YCCK => jpeg_encoder::ColorType::Ycck,
        c => panic!("Unsupported colorspace {c:?}"),
    };

    encoder
        .encode(&pixels, info.width as u16, info.height as u16, colorspace)
        .unwrap();

    let mut jpeg = gufo::jpeg::Jpeg::new(buf.into_inner()).unwrap();
    let new_jpeg = Jpeg::new(out_buf).unwrap();

    jpeg.replace_image_data(&new_jpeg).unwrap();

    check_images_eq(jpeg.into_inner(), reference);
}

fn check_images_eq(jpeg: Vec<u8>, reference: Vec<u8>) {
    let image = image::load_from_memory(&jpeg).unwrap();
    let reference_image = image::load_from_memory(&reference).unwrap();

    let similarity =
        image_compare::rgba_hybrid_compare(&image.into_rgba8(), &reference_image.into_rgba8())
            .unwrap();

    assert!(similarity.score > 0.95);
}
