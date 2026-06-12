use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Shows an output channel in the workbench via Vine IPC.
pub async fn OutputChannelShow(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OutputChannelShow::OutputChannelShow(Service, Parameter).await;
}
