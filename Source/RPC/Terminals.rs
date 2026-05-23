
//! Terminal-emulation RPC service. Placeholder for the Cocoon PTY +
//! shell-integration roadmap. Status: not yet wired; all exports are
//! cfg-gated behind the `terminals` feature.

#[cfg(feature = "terminals")]
pub struct Struct;

#[cfg(feature = "terminals")]
impl Struct {
	pub fn new() -> Self { Struct }
}

#[cfg(feature = "terminals")]
impl Default for Struct {
	fn default() -> Self { Self::new() }
}
