use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Appends content to a specific output channel via Vine IPC.
pub async fn OutputChannelAppend(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OutputChannelAppend::OutputChannelAppend(Service, Parameter).await;
}
