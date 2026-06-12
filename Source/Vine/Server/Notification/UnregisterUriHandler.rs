use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Unregisters uri handler.
pub async fn UnregisterUriHandler(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::UnregisterUriHandler::UnregisterUriHandler(Service, Parameter).await;
}
