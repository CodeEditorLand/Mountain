use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Manages debug session lifecycle notifications via Vine IPC.
pub async fn DebugLifecycle(Service:&MountainVinegRPCService, MethodName:&str, Parameter:&Value) {
	::Vine::Server::Notification::DebugLifecycle::DebugLifecycle(Service, MethodName, Parameter).await;
}
