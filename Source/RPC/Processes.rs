//! Child-process RPC service. Placeholder for spawn + stdio + signal
//! handling for Cocoon. Status: not yet wired; all exports are cfg-gated
//! behind the `child-processes` feature.

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
