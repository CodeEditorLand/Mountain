use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Registers an SCM provider via Vine IPC.
pub async fn RegisterScmProvider(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::RegisterScmProvider::RegisterScmProvider(Service, Parameter).await;
}
