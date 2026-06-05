# Gufo Exif

Gufo exif is a native Rust crate to read and edit Exif metadata.

This crate is specifically focused on editing Exif data while preserving the existing structure as much as possible. Every edit operation tries to only updates the raw data as much as necessary.

## Existing Crates

| Crate                                                 | Info                         | Comment        |
|-------------------------------------------------------|------------------------------|----------------|
| [exif-rs](https://crates.io/crates/exif-rs)           | Native, read only            | Abandoned      |
| [exif-sys](https://crates.io/crates/exif-sys)         | FFI bindings for libexif     | Abandoned      |
| [exif](https://crates.io/crates/exif)                 | Save binding for exif-sys    | Abandoned      |
| [gexiv2](https://crates.io/crates/gexiv2-sys)         | FFI bindings for gexiv2      |                |
| [imagemeta](https://crates.io/crates/imagemeta)       | Native                       | Abandoned      |
| [kamadak-exif](https://crates.io/crates/kamadak-exif) | Native, experimental writing | Quasi standard |
| [little\_exif](https://crates.io/crates/little_exif)  | Native                       |                |
| [peck-exif](https://crates.io/crates/peck-exif)       | Native, read only            |                |
| [rexif](https://crates.io/crates/rexif)               | Native                       |                |
| [rexiv2](https://crates.io/crates/rexiv2)             | Save bindings for gexiv2     |                |

## Relevant Standards

- [Exif Version 0.3](https://archive.org/details/exif-specs-3.0-dc-008-translation-2023-e/)
- [Exif metadata for XMP 2024](https://www.cipa.jp/std/documents/download_e.html?CIPA_DC-010-2024_E)
