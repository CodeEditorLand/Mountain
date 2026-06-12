use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Creates a new output channel via Vine IPC.
pub async fn OutputCreate(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OutputCreate::OutputCreate(Service, Parameter).await;
}
