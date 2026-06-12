use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Outputs channel show.
pub async fn OutputChannelShow(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OutputChannelShow::OutputChannelShow(Service, Parameter).await;
}
