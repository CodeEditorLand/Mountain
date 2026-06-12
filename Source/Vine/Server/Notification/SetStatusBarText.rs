use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Sets the status bar text via Vine IPC.
pub async fn SetStatusBarText(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::SetStatusBarText::SetStatusBarText(Service, Parameter).await;
}
