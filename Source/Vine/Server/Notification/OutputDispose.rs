use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Outputs dispose.
pub async fn OutputDispose(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::OutputDispose::OutputDispose(Service, Parameter).await;
}
