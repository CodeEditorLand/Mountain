use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn WindowShowMessage(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::WindowShowMessage::WindowShowMessage(Service, Parameter).await;
}
