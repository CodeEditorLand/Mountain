use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn UnregisterDebugAdapter(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::UnregisterDebugAdapter::UnregisterDebugAdapter(Service, Parameter).await;
}
