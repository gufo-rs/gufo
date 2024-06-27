use std::io::{Cursor, Read, Seek, SeekFrom};

use crate::utils;

pub const EXIF_IDENTIFIER_STRING: &[u8] = b"Exif\0\0";
pub const XMP_IDENTIFIER_STRING: &[u8] = b"http://ns.adobe.com/xap/1.0/\0";

#[derive(Clone, Debug)]
pub struct ObjectBox<'a> {
    box_type: BoxType,
    pos: u64,
    data: &'a [u8],
}

impl<'a> ObjectBox<'a> {
    pub fn box_type(&self) -> BoxType {
        self.box_type
    }

    pub fn pos(&self) -> u64 {
        self.pos
    }

    pub fn data_pos(&self) -> u64 {
        self.pos + 2
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }
}

pub struct Document<'a> {
    boxes: Vec<ObjectBox<'a>>,
}

impl<'a> Document<'a> {
    pub fn new(data: &'a [u8], skip_bytes: u64) -> Self {
        let boxes = Self::find_boxes(data, skip_bytes);
        Self { boxes }
    }

    /// List all segments with the given marker
    pub fn boxes_type(&self, box_type: BoxType) -> impl Iterator<Item = &ObjectBox<'a>> {
        self.boxes.iter().filter(move |x| x.box_type == box_type)
    }

    fn find_boxes(data: &[u8], skip_bytes: u64) -> Vec<ObjectBox> {
        let mut source = Cursor::new(data);

        source.seek(SeekFrom::Start(skip_bytes)).unwrap();

        let buf_32 = &mut [0; 4];
        let buf_64 = &mut [0; 8];
        let mut eof = false;

        let mut boxes = Vec::new();
        loop {
            let box_start_pos = source.stream_position().unwrap();

            // Read len
            source.read_exact(buf_32).unwrap();
            let mut size = u32::from_be_bytes(*buf_32) as u64;

            // Read type
            source.read_exact(buf_32).unwrap();
            let box_type = BoxType::from(*buf_32);

            //eprintln!("Found tag {buf_32:x?} {}", String::from_utf8_lossy(buf_32));
            dbg!(box_type);

            if size == 1 {
                source.read_exact(buf_64).unwrap();
                size = u64::from_be_bytes(*buf_64);
            } else if size == 0 {
                size = data.len() as u64 - box_start_pos;
                eof = true;
            }

            let mut pos = source.stream_position().unwrap();

            let unalignment = (pos - box_start_pos) % 8;
            dbg!(unalignment);
            if unalignment > 0 {
                pos += 8 - unalignment;
            }

            let data_end = box_start_pos as usize + size as usize;

            let object_box = ObjectBox {
                box_type,
                pos,
                data: data.as_ref().get((pos) as usize..data_end).unwrap(),
            };

            boxes.push(object_box);

            if eof || data_end == data.len() {
                break;
            }

            source.seek(SeekFrom::Start(box_start_pos + size)).unwrap();
        }

        boxes
    }
}

utils::convertible_enum_binary!(
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum BoxType {
        File = b"ftyp",
        Brotli = b"brob",
        JxlImage = b"jxlc",
        JxlImagePartial = b"jxlp",
        JpegBitstreamReconstructionData = b"jbrd",
    }
);
