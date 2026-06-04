use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn OpenExternal(Service:&MountainVinegRPCService, Parameter:&Value) {

	::Vine::Server::Notification::OpenExternal::OpenExternal(Service, Parameter).await;
}
