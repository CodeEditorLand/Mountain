use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Triggers a tree view refresh via Vine IPC.
pub async fn TreeRefresh(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::TreeRefresh::TreeRefresh(Service, Parameter).await;
}
