#![allow(non_snake_case)]
//! `unregister_file_system_provider` — disposes a scheme-bound FS provider.
//! Logs the scheme so routing mismatches are visible after disposal.

use serde_json::Value;

use super::Support::UnregisterByHandle::UnregisterByHandle;
use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn UnregisterFileSystemProvider(Service:&MountainVinegRPCService, Parameter:&Value) {
	let Scheme = Parameter.get("scheme").and_then(Value::as_str).unwrap_or("");
	dev_log!("provider-register", "[ProviderUnregister] file_system scheme={}", Scheme);
	UnregisterByHandle(Service, Parameter, "file_system");
}
