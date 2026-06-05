mod error;
mod exif;
pub mod structure;

pub use error::*;
pub use exif::{ExifInternal, Storage};

pub type Exif = ExifInternal<'static, exif::OwnedStore>;
pub type ExifMutBorrowed<'a> = ExifInternal<'a, exif::MutBorrowedStore<'a>>;
pub use structure::Typed;
