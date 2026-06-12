use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Unregisters remote authority resolver.
pub async fn UnregisterRemoteAuthorityResolver(Service:&MountainVinegRPCService, Parameter:&Value) {
	::Vine::Server::Notification::UnregisterRemoteAuthorityResolver::UnregisterRemoteAuthorityResolver(
		Service, Parameter,
	)
	.await;
}
