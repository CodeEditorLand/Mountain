use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Unregisters a provider by its handle, delegating to the canonical Vine
/// `UnregisterByHandle` implementation.
pub fn UnregisterByHandle(Service:&MountainVinegRPCService, Parameter:&Value, TypeName:&str) {
	::Vine::Server::Notification::Support::UnregisterByHandle::UnregisterByHandle(Service, Parameter, TypeName);
}
