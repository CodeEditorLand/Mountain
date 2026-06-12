use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Progresss end.
pub async fn ProgressEnd(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::ProgressEnd::ProgressEnd(Service, Parameter).await;
}
