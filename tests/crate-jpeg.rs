use std::io::{Cursor, Seek};

use gufo_common::field;
use gufo_exif::structure::Typed;
use gufo_jpeg::Jpeg;
use zune_jpeg::zune_core::colorspace;

#[test]
fn jpeg_exif() {
    let data = std::fs::read("exif-xmp.jpg").unwrap();
    let jpeg = gufo_jpeg::Jpeg::new(data).unwrap();

    let exif = gufo_exif::ExifOwned::for_vec(jpeg.exif_data().next().unwrap().to_vec()).unwrap();

    assert_eq!(
        exif.orientation(),
        Some(gufo_common::orientation::Orientation::Id)
    );

    assert_eq!(exif.model(), Some(String::from("iPhone 6")));
}

#[test]
fn jpeg_xmp() {
    let data = std::fs::read("exif-xmp.jpg").unwrap();
    let jpeg = gufo_jpeg::Jpeg::new(data).unwrap();

    let xmp = gufo_xmp::Xmp::new(jpeg.xmp_data().next().unwrap().to_vec()).unwrap();

    assert_eq!(
        xmp.lookup_generic(gufo_xmp::Tag::new(
            gufo_common::xmp::Namespace::Xmp,
            "CreatorTool".into()
        )),
        Some("GIMP 2.10")
    );
}

#[test]
fn jpeg_rotate() {
    let data = std::fs::read("exif-xmp.jpg").unwrap();

    let jpeg = gufo_jpeg::Jpeg::new(data).unwrap();

    let mut exif =
        gufo_exif::ExifOwned::for_vec(jpeg.exif_data().next().unwrap().to_vec()).unwrap();
    let jpeg_exif_segment_pos = jpeg.exif_segments().map(|x| x.data_pos()).next().unwrap();

    let current_orientation = exif.orientation().unwrap();

    let mut data = jpeg.into_inner();

    let new_rotation =
        dbg!(current_orientation.rotate()) + gufo_common::orientation::Rotation::_180;
    let new_orientation =
        gufo_common::orientation::Orientation::new(current_orientation.mirror(), new_rotation);

    let diff = exif
        .update_entry_diff(
            field::Orientation.into(),
            Typed::Short(vec![new_orientation as u16]),
        )
        .unwrap();

    for (pos, val) in diff {
        // Position is relative to the exif data. Make it absolute by adding the
        // position of exif data in the JPEG.
        let abs_pos = gufo::jpeg::EXIF_IDENTIFIER_STRING.len() + jpeg_exif_segment_pos + pos;
        data[abs_pos] = val;
    }

    let jpeg = gufo_jpeg::Jpeg::new(data).unwrap();
    let exif = gufo_exif::ExifOwned::for_vec(jpeg.exif_data().next().unwrap().to_vec()).unwrap();
    assert_eq!(
        exif.orientation(),
        Some(gufo_common::orientation::Orientation::Rotation180)
    );
}

#[test]
pub fn jpeg_re_encode_ycbcr() {
    re_encode(
        std::fs::read("test-images/images/color/color.jpg").unwrap(),
        std::fs::read("test-images/images/color.png").unwrap(),
    );
}

#[test]
pub fn jpeg_re_encode_ycck() {
    re_encode(
        std::fs::read("test-images/images/color/color_cmyk.jpg").unwrap(),
        std::fs::read("test-images/images/color.png").unwrap(),
    );
}

#[test]
pub fn jpeg_re_encode_luma() {
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

    let decoder_options = zune_jpeg::zune_core::options::DecoderOptions::new_fast();
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
