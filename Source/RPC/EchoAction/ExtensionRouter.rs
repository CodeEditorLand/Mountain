
//! Pairs an extension identifier with the host that owns it. Used by
//! `EchoActionServer` to scope priority/telemetry when more than one
//! extension host is active (Grove + Cocoon).

use std::sync::Arc;

use crate::RPC::EchoAction::ExtensionHostRegistry;

pub struct Struct {
	Registry:Arc<ExtensionHostRegistry::Struct>,
}

impl Struct {
	pub fn new(Registry:Arc<ExtensionHostRegistry::Struct>) -> Self { Self { Registry } }

	pub async fn HostFor(&self, ExtensionIdentifier:&str) -> Option<String> {
		self.Registry.Resolve(ExtensionIdentifier).await
	}
}
