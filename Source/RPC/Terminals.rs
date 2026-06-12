//! Terminal-emulation RPC service. Placeholder for PTY +
//! shell integration. Status: not yet wired; all exports are
//! cfg-gated behind the `terminals` feature.
/// Terminal-emulation service handle.
#[cfg(feature = "terminals")]
pub struct Struct;

/// Creates a new `Struct`.
#[cfg(feature = "terminals")]
impl Struct {
	pub fn new() -> Self { Struct }
}

#[cfg(feature = "terminals")]
impl Default for Struct {
	fn default() -> Self { Self::new() }
}
