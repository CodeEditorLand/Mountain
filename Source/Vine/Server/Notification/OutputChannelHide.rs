use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Hides an output channel from the workbench via Vine IPC.
pub async fn OutputChannelHide(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OutputChannelHide::OutputChannelHide(Service, Parameter).await;
}
