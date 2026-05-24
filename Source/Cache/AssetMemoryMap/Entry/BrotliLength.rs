//! `Entry::BrotliLength`

use super::Struct;
use memmap2::Mmap;

pub fn Fn(This:&Struct) -> Option<usize> { This.Brotli.as_ref().map(|M| M.len()) }
