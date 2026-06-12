use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Updates an active progress indicator via Vine IPC.
pub async fn ProgressUpdate(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::ProgressUpdate::ProgressUpdate(Service, Parameter).await;
}
