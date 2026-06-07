use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn UnregisterAuthenticationProvider(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::UnregisterAuthenticationProvider::UnregisterAuthenticationProvider(
		Service, Parameter,
	)
	.await;
}
