#![allow(non_snake_case)]

//! Window-management RPC service. Placeholder for the Grove + Cocoon
//! extension-host roadmap (window/document/webview lifecycle).
//! Cfg-gated `pub struct Struct`. TODO: zero callers as of 2026-05-02.

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
