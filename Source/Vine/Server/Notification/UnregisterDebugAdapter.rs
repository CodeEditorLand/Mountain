use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Unregisters a debug adapter via Vine IPC.
pub async fn UnregisterDebugAdapter(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::UnregisterDebugAdapter::UnregisterDebugAdapter(Service, Parameter).await;
}
