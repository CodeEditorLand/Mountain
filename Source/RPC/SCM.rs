//! Source-control-management RPC service. Placeholder for git repo
//! discovery, change tracking, commit/push operations.
/// Source-control-management service handle.
#[cfg(feature = "scm-support")]
pub struct Struct;

/// Creates a new `Struct`.
#[cfg(feature = "scm-support")]
impl Struct {
	pub fn new() -> Self { Struct }
}

#[cfg(feature = "scm-support")]
impl Default for Struct {
	fn default() -> Self { Self::new() }
}
