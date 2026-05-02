#![allow(non_snake_case)]

//! Child-process RPC service. Placeholder for `spawn` + stdio + signal
//! handling for Cocoon. TODO: zero callers as of 2026-05-02.

#[cfg(feature = "child-processes")]
pub struct Struct;

#[cfg(feature = "child-processes")]
impl Struct {
	pub fn new() -> Self { Struct }
}

#[cfg(feature = "child-processes")]
impl Default for Struct {
	fn default() -> Self { Self::new() }
}
