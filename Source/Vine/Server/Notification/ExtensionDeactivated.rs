use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn ExtensionDeactivated(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::ExtensionDeactivated::ExtensionDeactivated(Service, Parameter).await;
}
