use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn UnregisterTaskProvider(Service:&MountainVinegRPCService, Parameter:&Value) {

	::Vine::Server::Notification::UnregisterTaskProvider::UnregisterTaskProvider(Service, Parameter).await;
}
