//! Debug-Adapter-Protocol RPC service. Placeholder for the Cocoon DAP
//! roadmap. Status: not yet wired; all exports are cfg-gated behind the
//! `debug-protocol` feature.

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
