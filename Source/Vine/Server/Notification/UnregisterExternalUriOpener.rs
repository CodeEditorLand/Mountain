use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn UnregisterExternalUriOpener(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::UnregisterExternalUriOpener::UnregisterExternalUriOpener(Service, Parameter).await;
}
