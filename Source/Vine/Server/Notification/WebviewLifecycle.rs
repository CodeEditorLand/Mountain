use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn WebviewLifecycle(Service:&MountainVinegRPCService, MethodName:&str, Parameter:&Value) {

	::Vine::Server::Notification::WebviewLifecycle::WebviewLifecycle(Service, MethodName, Parameter).await;
}
