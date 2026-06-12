use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Progresss update.
pub async fn ProgressUpdate(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::ProgressUpdate::ProgressUpdate(Service, Parameter).await;
}
