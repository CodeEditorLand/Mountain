#![allow(non_snake_case)]

//! Debug-Adapter-Protocol RPC service. Placeholder for the Cocoon DAP
//! roadmap. TODO: zero callers as of 2026-05-02.

#[cfg(feature = "debug-protocol")]
pub struct Struct;

#[cfg(feature = "debug-protocol")]
impl Struct {
	pub fn new() -> Self { Struct }
}

#[cfg(feature = "debug-protocol")]
impl Default for Struct {
	fn default() -> Self { Self::new() }
}
