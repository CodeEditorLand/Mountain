use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Decorations type lifecycle.
pub async fn DecorationTypeLifecycle(Service:&MountainVinegRPCService, MethodName:&str, Parameter:&Value) {
	::Vine::Server::Notification::DecorationTypeLifecycle::DecorationTypeLifecycle(Service, MethodName, Parameter)
		.await;
}
