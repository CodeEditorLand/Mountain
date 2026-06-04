use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn StatusBarMessage(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::StatusBarMessage::StatusBarMessage(Service, Parameter).await;
}
