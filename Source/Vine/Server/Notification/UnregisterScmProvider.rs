use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn UnregisterScmProvider(Service:&MountainVinegRPCService, Parameter:&Value) {

	::Vine::Server::Notification::UnregisterScmProvider::UnregisterScmProvider(Service, Parameter).await;
}
