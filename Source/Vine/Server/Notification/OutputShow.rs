use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Outputs show.
pub async fn OutputShow(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OutputShow::OutputShow(Service, Parameter).await;
}
