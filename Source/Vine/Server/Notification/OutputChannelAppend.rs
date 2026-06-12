use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Outputs channel append.
pub async fn OutputChannelAppend(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OutputChannelAppend::OutputChannelAppend(Service, Parameter).await;
}
