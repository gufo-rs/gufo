use std::collections::HashMap;

use super::{TagIfd, Type};
use crate::error::Result;

impl super::ExifRaw {
    pub fn debug_dump(&mut self) -> String {
        let mut out = String::new();

        let mut ifd_locations: HashMap<_, _> = self
            .ifd_locations
            .clone()
            .into_iter()
            .map(|(ifd, ifd_location)| (ifd, Some(ifd_location)))
            .collect();

        for (tagifd, _) in self.locations.iter() {
            ifd_locations.entry(tagifd.ifd).or_default();
        }

        for (ifd, ifd_location) in ifd_locations {
            out.push_str(&format!("\n{ifd:?} - Defined {ifd_location:?}\n"));
            out.push_str("------------------------------\n");
            for (tagifd, entries) in self.locations.clone() {
                if tagifd.ifd == ifd {
                    let tag = tagifd.tag.0;
                    for entry in entries {
                        let name = gufo_common::exif::lookup_tag_name(tagifd).unwrap_or("Unknown");
                        out.push_str(&format!(
                            "{name} {tag} (0x{tag:X}) {:?} ({}x): {:?}\n",
                            entry.data_type, entry.count, entry.value_offset
                        ));
                        out.push_str(&self.debug_dump_entry(tagifd));
                        out.push('\n');
                    }
                }
            }
        }

        out
    }

    pub fn debug_dump_entry(&mut self, tagifd: TagIfd) -> String {
        fn show(x: Result<Option<impl ToString>>) -> String {
            if let Ok(Some(x)) = x {
                x.to_string()
            } else {
                format!("{:?}", x.err())
            }
        }

        if let Some(entry) = self.lookup_entry(tagifd) {
            match entry.data_type {
                Type::Ascii | Type::Utf8 => show(self.lookup_string(tagifd)),
                Type::Short if entry.count == 1 => show(self.lookup_short(tagifd)),
                Type::Long if entry.count == 1 => show(self.lookup_long(tagifd)),
                Type::Rational if entry.count == 1 => show(
                    self.lookup_rational(tagifd)
                        .map(|x: Option<(u32, u32)>| x.map(|(x, y)| format!("{x}/{y}"))),
                ),
                _ => format!("Unknown type {:?}", entry.data_type),
            }
        } else {
            String::from("Not found")
        }
    }
}
