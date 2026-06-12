use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Extensions host message.
pub async fn ExtensionHostMessage(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::ExtensionHostMessage::ExtensionHostMessage(Service, Parameter).await;
}
