use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Outputs channel create.
pub async fn OutputChannelCreate(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OutputChannelCreate::OutputChannelCreate(Service, Parameter).await;
}
