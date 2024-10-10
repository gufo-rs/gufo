#![doc = include_str!("../README.md")]

mod chunk;
mod chunk_type;
mod error;
mod png;

pub use chunk::*;
pub use chunk_type::*;
pub use error::*;
use gufo_common::error::ErrorWithData;
pub use png::*;

#[macro_export]
macro_rules! remove_chunk {
    ($png:ident, $chunk:expr) => {{
        let raw_chunk = $chunk.unsafe_raw_chunk();
        $png.remove_chunk(raw_chunk)
    }};
}
