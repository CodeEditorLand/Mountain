use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Progresss complete.
pub async fn ProgressComplete(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::ProgressComplete::ProgressComplete(Service, Parameter).await;
}
