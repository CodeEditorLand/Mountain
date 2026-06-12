use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Webviews dispose.
pub async fn WebviewDispose(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::WebviewDispose::WebviewDispose(Service, Parameter).await;
}
