use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Unregisters authentication provider.
pub async fn UnregisterAuthenticationProvider(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::UnregisterAuthenticationProvider::UnregisterAuthenticationProvider(
		Service, Parameter,
	)
	.await;
}
