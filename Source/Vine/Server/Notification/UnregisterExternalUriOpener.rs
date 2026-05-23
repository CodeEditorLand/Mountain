//! `unregister_external_uri_opener` - disposes an external URI opener handle.

use serde_json::Value;

use super::Support::UnregisterByHandle::UnregisterByHandle;
use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn UnregisterExternalUriOpener(Service:&MountainVinegRPCService, Parameter:&Value) {
	UnregisterByHandle(Service, Parameter, "external_uri_opener");
}
