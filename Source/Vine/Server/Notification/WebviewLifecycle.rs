use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Manages webview panel lifecycle via Vine IPC.
pub async fn WebviewLifecycle(Service:&MountainVinegRPCService, MethodName:&str, Parameter:&Value) {
	::Vine::Server::Notification::WebviewLifecycle::WebviewLifecycle(Service, MethodName, Parameter).await;
}
