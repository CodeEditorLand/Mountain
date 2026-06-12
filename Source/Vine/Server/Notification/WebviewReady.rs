use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Webviews ready.
pub async fn WebviewReady(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::WebviewReady::WebviewReady(Service, Parameter).await;
}
