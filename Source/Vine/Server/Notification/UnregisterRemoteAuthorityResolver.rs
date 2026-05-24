//! `unregister_remote_authority_resolver` - disposes a remote authority
//! resolver handle.

use serde_json::Value;

use super::Support::Fn::Fn;
use crate::Vine::Server::MountainVinegRPCService::Struct;

pub async fn Fn(Service:&MountainVinegRPCService, Parameter:&Value) {
	UnregisterByHandle(Service, Parameter, "remote_authority_resolver");
}
