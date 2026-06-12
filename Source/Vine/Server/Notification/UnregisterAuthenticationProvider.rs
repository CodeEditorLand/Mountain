use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Unregisters an authentication provider via Vine IPC.
pub async fn UnregisterAuthenticationProvider(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::UnregisterAuthenticationProvider::UnregisterAuthenticationProvider(
		Service, Parameter,
	)
	.await;
}
