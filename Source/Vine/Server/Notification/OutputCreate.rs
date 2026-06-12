use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Outputs create.
pub async fn OutputCreate(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OutputCreate::OutputCreate(Service, Parameter).await;
}
