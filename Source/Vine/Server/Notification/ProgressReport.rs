use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Reports a progress update via Vine IPC.
pub async fn ProgressReport(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::ProgressReport::ProgressReport(Service, Parameter).await;
}
