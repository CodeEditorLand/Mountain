use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn ProgressComplete(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::ProgressComplete::ProgressComplete(Service, Parameter).await;
}
