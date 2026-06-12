//! Tracks which extension host owns which extension id. Populated from
//! `$deltaExtensions` + `InitExtensionHost` payloads; read by
//! `ExtensionRouter` when a request needs to be routed to a specific host.
use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

/// Registry mapping extension identifiers to host identifiers.
pub struct Struct {
	Hosts:Arc<RwLock<HashMap<String, String>>>,
}

/// Creates a new empty `ExtensionHostRegistry`.
impl Struct {
	pub fn new() -> Self { Self { Hosts:Arc::new(RwLock::new(HashMap::new())) } }

	/// Record an extension-to-host mapping.
	pub async fn Record(&self, ExtensionIdentifier:String, HostIdentifier:String) {
		self.Hosts.write().await.insert(ExtensionIdentifier, HostIdentifier);
	}

	/// Remove an extension-to-host mapping.
	pub async fn Forget(&self, ExtensionIdentifier:&str) { self.Hosts.write().await.remove(ExtensionIdentifier); }

	/// Resolve the host identifier for a given extension identifier.
	pub async fn Resolve(&self, ExtensionIdentifier:&str) -> Option<String> {
		self.Hosts.read().await.get(ExtensionIdentifier).cloned()
	}

	/// Return the number of registered extension-to-host mappings.
	pub async fn Count(&self) -> usize { self.Hosts.read().await.len() }
}

impl Default for Struct {
	fn default() -> Self { Self::new() }
}
