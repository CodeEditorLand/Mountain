use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Dispatches status bar item lifecycle events via Vine IPC.
pub async fn StatusBarLifecycle(Service:&MountainVinegRPCService, MethodName:&str, Parameter:&Value) {
	::Vine::Server::Notification::StatusBarLifecycle::StatusBarLifecycle(Service, MethodName, Parameter).await;
}
