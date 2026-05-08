#![allow(non_snake_case)]

//! Diagnostic snapshot of the canonical-path cache.

#[derive(Debug, Clone, Copy)]
pub struct Struct {
	pub Entries:usize,

	pub WeightedSize:usize,
}
