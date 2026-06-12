use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Starts a progress indicator via Vine IPC.
pub async fn ProgressStart(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::ProgressStart::ProgressStart(Service, Parameter).await;
}
