use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Displays a status bar message via Vine IPC.
pub async fn StatusBarMessage(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::StatusBarMessage::StatusBarMessage(Service, Parameter).await;
}
