use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn UpdateScmGroup(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::UpdateScmGroup::UpdateScmGroup(Service, Parameter).await;
}
