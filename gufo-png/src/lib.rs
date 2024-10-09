#![doc = include_str!("../README.md")]

mod chunk;
mod chunk_type;
mod error;
mod png;

pub use error::*;
pub use png::*;

pub use chunk::*;
pub use chunk_type::*;
use gufo_common::error::ErrorWithData;

#[macro_export]
macro_rules! remove_chunk {
    ($png:ident, $chunk:expr) => {{
        let raw_chunk = $chunk.unsafe_raw_chunk();
        $png.remove_chunk(raw_chunk)
    }};
}
