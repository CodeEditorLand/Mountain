use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Manages decoration type lifecycle events via Vine IPC.
pub async fn DecorationTypeLifecycle(Service:&MountainVinegRPCService, MethodName:&str, Parameter:&Value) {
	::Vine::Server::Notification::DecorationTypeLifecycle::DecorationTypeLifecycle(Service, MethodName, Parameter)
		.await;
}
