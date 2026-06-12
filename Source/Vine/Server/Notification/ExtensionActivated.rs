use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Forwards extension activated events via Vine IPC.
pub async fn ExtensionActivated(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::ExtensionActivated::ExtensionActivated(Service, Parameter).await;
}
