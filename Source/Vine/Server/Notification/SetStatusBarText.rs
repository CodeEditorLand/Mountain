use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Sets status bar text.
pub async fn SetStatusBarText(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::SetStatusBarText::SetStatusBarText(Service, Parameter).await;
}
