use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn ProgressStart(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::ProgressStart::ProgressStart(Service, Parameter).await;
}
