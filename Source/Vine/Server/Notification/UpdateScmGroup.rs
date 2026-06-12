use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Updates scm group.
pub async fn UpdateScmGroup(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::UpdateScmGroup::UpdateScmGroup(Service, Parameter).await;
}
