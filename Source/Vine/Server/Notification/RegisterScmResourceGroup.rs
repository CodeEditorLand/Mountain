use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Registers scm resource group.
pub async fn RegisterScmResourceGroup(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::RegisterScmResourceGroup::RegisterScmResourceGroup(Service, Parameter).await;
}
