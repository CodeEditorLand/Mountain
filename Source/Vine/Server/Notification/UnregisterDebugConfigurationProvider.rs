use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn UnregisterDebugConfigurationProvider(Service:&MountainVinegRPCService, Parameter:&Value) {

	::Vine::Server::Notification::UnregisterDebugConfigurationProvider::UnregisterDebugConfigurationProvider(
		Service, Parameter,
	)
	.await;
}
