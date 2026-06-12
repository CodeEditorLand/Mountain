//! Pairs an extension identifier with the host that owns it. Used by
//! `EchoActionServer` to scope priority/telemetry when more than one
//! extension host is active.
use std::sync::Arc;

use crate::RPC::EchoAction::ExtensionHostRegistry;

/// Router pairing an extension to its owning host.
pub struct Struct {
	Registry:Arc<ExtensionHostRegistry::Struct>,
}

/// Creates a new `ExtensionRouter` backed by the given registry.
impl Struct {
	pub fn new(Registry:Arc<ExtensionHostRegistry::Struct>) -> Self { Self { Registry } }

	/// Resolve the host identifier for a given extension identifier.
	pub async fn HostFor(&self, ExtensionIdentifier:&str) -> Option<String> {
		self.Registry.Resolve(ExtensionIdentifier).await
	}
}
