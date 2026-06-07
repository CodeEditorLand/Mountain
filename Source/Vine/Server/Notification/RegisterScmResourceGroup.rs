use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn RegisterScmResourceGroup(Service:&MountainVinegRPCService, Parameter:&Value) {

	::Vine::Server::Notification::RegisterScmResourceGroup::RegisterScmResourceGroup(Service, Parameter).await;
}
