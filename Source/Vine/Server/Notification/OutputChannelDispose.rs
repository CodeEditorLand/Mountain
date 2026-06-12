use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Disposes an output channel via Vine IPC.
pub async fn OutputChannelDispose(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OutputChannelDispose::OutputChannelDispose(Service, Parameter).await;
}
