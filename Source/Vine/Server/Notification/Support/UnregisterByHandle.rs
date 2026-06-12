use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Unregisters by handle.
pub fn UnregisterByHandle(Service:&MountainVinegRPCService, Parameter:&Value, TypeName:&str) {
	::Vine::Server::Notification::Support::UnregisterByHandle::UnregisterByHandle(Service, Parameter, TypeName);
}
