use gufo_jxl::*;
fn main() {
    println!("Hello, world!");
    let path = "gufo-exif/tests/test-images/images/exif/exif.jxl";
    //let path = "/home/herold/Downloads/loupetest/railroad/out.jxl";
    //let path =
    //    "gufo-exif/tests/test-images/images/color-exif-orientation/color-iccp-pro-rotated-90.jxl";
    let data = std::fs::read(path).unwrap();

    let document = Document::new(&data).unwrap();

    //let x = bs.read_bundle();

    /*
    let document = gufo_common::isobmff::Document::new(&data, 12);
    */
    let b = document
        .document
        .boxes_type(gufo_common::isobmff::BoxType::JxlImagePartial)
        .next()
        .unwrap();
    dbg!(&b.data()[0..10]);

    let mut bs = jxl_bitstream::Bitstream::new(&b.data()[4..10]);
    dbg!(bs.read_bits(8));
    dbg!(bs.read_bits(8));
    //dbg!(bs.read_bits(1));
    //eprintln!("{:b}", bs.read_bits(8).unwrap());
    let x: jxl_image::SizeHeader = bs.read_bundle().unwrap();
    dbg!(x);

    let mut i = document.image_data().unwrap();
    let h = Header::parse(&mut i);
    dbg!(h);
}
