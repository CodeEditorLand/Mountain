use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Clears all content from a specific output channel via Vine IPC.
pub async fn OutputChannelClear(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OutputChannelClear::OutputChannelClear(Service, Parameter).await;
}
