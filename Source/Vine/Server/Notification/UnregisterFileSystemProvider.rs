use serde_json::Value;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn UnregisterFileSystemProvider(Service:&MountainVinegRPCService, Parameter:&Value) {

	// Preserve scheme in log so routing mismatches are visible after disposal.
	let Scheme = Parameter.get("scheme").and_then(Value::as_str).unwrap_or("");

	dev_log!("provider-register", "[ProviderUnregister] file_system scheme={}", Scheme);

	::Vine::Server::Notification::UnregisterFileSystemProvider::UnregisterFileSystemProvider(Service, Parameter).await;
}
