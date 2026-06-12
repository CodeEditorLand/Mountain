use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Registers an SCM resource group via Vine IPC.
pub async fn RegisterScmResourceGroup(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::RegisterScmResourceGroup::RegisterScmResourceGroup(Service, Parameter).await;
}
