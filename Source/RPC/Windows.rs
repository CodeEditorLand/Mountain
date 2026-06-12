//! Window-management RPC service. Placeholder for window/document/webview
//! lifecycle. Status: not yet wired; all exports are cfg-gated behind `grove`
//! or `cocoon`.
/// Window-management service handle.
#[cfg(any(feature = "grove", feature = "cocoon"))]
pub struct Struct;

/// Creates a new `Struct`.
#[cfg(any(feature = "grove", feature = "cocoon"))]
impl Struct {
	pub fn new() -> Self { Struct }
}

#[cfg(any(feature = "grove", feature = "cocoon"))]
impl Default for Struct {
	fn default() -> Self { Self::new() }
}
