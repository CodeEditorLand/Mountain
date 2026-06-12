use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Posts a message to a webview panel via Vine IPC.
pub async fn WebviewPostMessage(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::WebviewPostMessage::WebviewPostMessage(Service, Parameter).await;
}
