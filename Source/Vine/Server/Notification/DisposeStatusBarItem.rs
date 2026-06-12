use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Disposes status bar item.
pub async fn DisposeStatusBarItem(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::DisposeStatusBarItem::DisposeStatusBarItem(Service, Parameter).await;
}
