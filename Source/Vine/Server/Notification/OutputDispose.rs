use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Disposes an output channel via Vine IPC.
pub async fn OutputDispose(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OutputDispose::OutputDispose(Service, Parameter).await;
}
