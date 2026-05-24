//! `unregister_external_uri_opener` - disposes an external URI opener handle.

use serde_json::Value;

use super::Support::Fn::Fn;
use crate::Vine::Server::MountainVinegRPCService::Struct;

pub async fn Fn(Service:&MountainVinegRPCService, Parameter:&Value) {
	UnregisterByHandle(Service, Parameter, "external_uri_opener");
}
