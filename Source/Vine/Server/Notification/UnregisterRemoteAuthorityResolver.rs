//! `unregister_remote_authority_resolver` - disposes a remote authority
//! resolver handle.

use serde_json::Value;

use super::Support::UnregisterByHandle::UnregisterByHandle;
use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn UnregisterRemoteAuthorityResolver(Service:&MountainVinegRPCService, Parameter:&Value) {
	UnregisterByHandle(Service, Parameter, "remote_authority_resolver");
}
