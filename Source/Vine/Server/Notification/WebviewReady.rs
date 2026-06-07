use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn WebviewReady(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::WebviewReady::WebviewReady(Service, Parameter).await;
}
