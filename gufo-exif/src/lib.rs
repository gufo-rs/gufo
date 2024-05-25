#![doc = include_str!("../README.md")]

pub mod error;
mod high_level;
pub mod internal;

pub use high_level::Exif;
