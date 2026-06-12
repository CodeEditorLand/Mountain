use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Ends a progress indicator via Vine IPC.
pub async fn ProgressEnd(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::ProgressEnd::ProgressEnd(Service, Parameter).await;
}
