use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Clears all content from an output channel via Vine IPC.
pub async fn OutputClear(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OutputClear::OutputClear(Service, Parameter).await;
}
