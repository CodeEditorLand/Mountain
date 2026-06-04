use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn UnregisterUriHandler(Service:&MountainVinegRPCService, Parameter:&Value) {

	::Vine::Server::Notification::UnregisterUriHandler::UnregisterUriHandler(Service, Parameter).await;
}
