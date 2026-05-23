//! `debug_configuration` provider-unregistration atom.

use serde_json::Value;

use super::Support::UnregisterByHandle::UnregisterByHandle;
use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn UnregisterDebugConfigurationProvider(Service:&MountainVinegRPCService, Parameter:&Value) {
	UnregisterByHandle(Service, Parameter, "debug_configuration");
}
