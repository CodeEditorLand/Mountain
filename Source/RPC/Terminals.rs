#![allow(non_snake_case)]

//! Terminal-emulation RPC service. Placeholder for the Cocoon PTY +
//! shell-integration roadmap. TODO: zero callers as of 2026-05-02.

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
