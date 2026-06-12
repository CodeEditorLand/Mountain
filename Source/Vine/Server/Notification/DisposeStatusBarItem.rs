use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Disposes a status bar item via Vine IPC.
pub async fn DisposeStatusBarItem(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::DisposeStatusBarItem::DisposeStatusBarItem(Service, Parameter).await;
}
