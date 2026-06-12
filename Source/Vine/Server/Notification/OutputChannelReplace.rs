use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Replaces the full content of an output channel via Vine IPC.
pub async fn OutputChannelReplace(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OutputChannelReplace::OutputChannelReplace(Service, Parameter).await;
}
