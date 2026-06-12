use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Unregisters an SCM provider via Vine IPC.
pub async fn UnregisterScmProvider(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::UnregisterScmProvider::UnregisterScmProvider(Service, Parameter).await;
}
