//! `Entry::AsSlice`

use memmap2::Mmap;

use super::Struct;

pub fn Fn(This:&Struct) -> &[u8] { &This.Mapping[..] }
