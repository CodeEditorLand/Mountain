#![allow(non_snake_case)]

//! Snapshot of asset-cache occupancy. Returned by
//! `Cache::AssetMemoryMap::Stats::Fn`.

#[derive(Debug, Clone, Copy)]
pub struct Struct {
	pub Entries:usize,

	pub BrotliEntries:usize,

	pub Bytes:usize,

	pub BrotliBytes:usize,
}
