//! Debug-Adapter-Protocol RPC service. Placeholder for the DAP.
//! Status: not yet wired; all exports are cfg-gated behind the `debug-protocol`
//! feature.
/// Debug-Adapter-Protocol service handle.
#[cfg(feature = "debug-protocol")]
pub struct Struct;

/// Creates a new `Struct`.
#[cfg(feature = "debug-protocol")]
impl Struct {
	pub fn new() -> Self { Struct }
}

#[cfg(feature = "debug-protocol")]
impl Default for Struct {
	fn default() -> Self { Self::new() }
}
