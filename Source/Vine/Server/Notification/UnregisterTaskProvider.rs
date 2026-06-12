use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Unregisters a task provider via Vine IPC.
pub async fn UnregisterTaskProvider(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::UnregisterTaskProvider::UnregisterTaskProvider(Service, Parameter).await;
}
