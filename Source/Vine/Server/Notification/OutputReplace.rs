use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Replaces the content of an output channel via Vine IPC.
pub async fn OutputReplace(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OutputReplace::OutputReplace(Service, Parameter).await;
}
