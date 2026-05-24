//! `Entry::AsBrotliSlice`

use memmap2::Mmap;

use super::Struct;

pub fn Fn(This:&Struct) -> Option<&[u8]> { This.Brotli.as_ref().map(|M| &M[..]) }
