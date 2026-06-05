#![doc=include_str!("../README.md")]

//! ## Usage
//!
//! The high level API in `Exif` provides simple access to commonly used
//! metadata.
//!
//! ```
//! let data = std::fs::read("example.jpg").unwrap();
//! let jpeg = gufo_jpeg::Jpeg::new(data).unwrap();
//! let raw_exif = jpeg.exif_data().next().unwrap().to_vec();
//!
//! let exif = gufo_exif::Exif::for_vec(raw_exif).unwrap();
//! assert_eq!(exif.model().as_deref(), Some("Canon EOS 400D DIGITAL"));
//! ```
//!
//! Other values can be looked up manually:
//!
//! ```
//! # let data = std::fs::read("example.jpg").unwrap();
//! # let jpeg = gufo_jpeg::Jpeg::new(data).unwrap();
//! # let raw_exif = jpeg.exif_data().next().unwrap().to_vec();
//! # let mut exif = gufo_exif::Exif::for_vec(raw_exif).unwrap();
//! let exposure_program = exif
//!     .document(|document| document.lookup_short(gufo_common::field::ExposureProgram.into()))
//!     .unwrap()
//!     .unwrap();
//! assert_eq!(exposure_program, 2);
//! ```
//!
//! Or even more manually:
//!
//! ```
//! # let data = std::fs::read("example.jpg").unwrap();
//! # let jpeg = gufo_jpeg::Jpeg::new(data).unwrap();
//! # let raw_exif = jpeg.exif_data().next().unwrap().to_vec();
//! # let mut exif = gufo_exif::Exif::for_vec(raw_exif).unwrap();
//! use gufo_common::exif::{IfdId, Tag, TagIfd};
//!
//! let exposure_program = exif
//!     .document(|document| {
//!         document.lookup(gufo_common::exif::TagIfd::new(Tag(0x8822), IfdId::Exif))
//!     })
//!     .unwrap()
//!     .unwrap();
//! assert_eq!(exposure_program, gufo_exif::Typed::Short(vec![2]));
//! ```
//!
//! Gufo exif also provides basic editing capabilities:
//! ```
//! # let data = std::fs::read("example.jpg").unwrap();
//! # let jpeg = gufo_jpeg::Jpeg::new(data).unwrap();
//! # let raw_exif = jpeg.exif_data().next().unwrap().to_vec();
//! # let mut exif = gufo_exif::Exif::for_vec(raw_exif).unwrap();
//! // Delete owner name from exif data and check it actually was present
//! assert!(matches!(
//!     exif.delete(gufo_common::field::CanonCameraOwnerName.into()),
//!     Ok(true)
//! ));
//!
//! // Change image orientation
//! exif.update_entry(
//!     gufo_common::field::Orientation.into(),
//!     gufo_exif::Typed::Short(vec![
//!         gufo_common::orientation::Orientation::Rotation180 as u16,
//!     ]),
//! );
//!
//! // Optain raw exif data that include the changes
//! let raw_exif = exif.serialize();
//! ```

mod error;
mod exif;
pub mod structure;

pub use error::Error;
pub use exif::{Exif, Storage};
pub use structure::Typed;

/// Version of [`Exif`] based on a [`Vec<u8>`].
pub type ExifOwned = Exif<'static, exif::OwnedStore>;
/// Version of [`Exif`] based on a [`&mut [u8]`](slice).
pub type ExifMutBorrowed<'a> = Exif<'a, exif::MutBorrowedStore<'a>>;
