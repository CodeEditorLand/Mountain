//! Window-management RPC service. Placeholder for the Grove + Cocoon
//! extension-host roadmap (window/document/webview lifecycle). Status:
//! not yet wired; all exports are cfg-gated behind `grove` or `cocoon`.

#[cfg(any(feature = "grove", feature = "cocoon"))]
pub struct Struct;

#[cfg(any(feature = "grove", feature = "cocoon"))]
impl Struct {
	pub fn new() -> Self { Struct }
}

#[cfg(any(feature = "grove", feature = "cocoon"))]
impl Default for Struct {
	fn default() -> Self { Self::new() }
}
