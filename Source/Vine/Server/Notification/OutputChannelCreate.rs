use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Creates a new output channel via Vine IPC.
pub async fn OutputChannelCreate(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OutputChannelCreate::OutputChannelCreate(Service, Parameter).await;
}
