use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Outputs channel clear.
pub async fn OutputChannelClear(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OutputChannelClear::OutputChannelClear(Service, Parameter).await;
}
