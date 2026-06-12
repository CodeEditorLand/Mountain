use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Notifies that a webview panel is ready via Vine IPC.
pub async fn WebviewReady(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::WebviewReady::WebviewReady(Service, Parameter).await;
}
