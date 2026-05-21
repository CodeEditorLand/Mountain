#![allow(non_snake_case)]
//! `unregister_uri_handler` — disposes a URI-handler provider handle.
//! Logs the optional bound scheme alongside the handle for traceability.

use serde_json::Value;

use super::Support::UnregisterByHandle::UnregisterByHandle;
use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn UnregisterUriHandler(Service:&MountainVinegRPCService, Parameter:&Value) {
	let Scheme = Parameter.get("scheme").and_then(Value::as_str).unwrap_or("");

	dev_log!("provider-register", "[ProviderUnregister] uri_handler scheme={}", Scheme);

	UnregisterByHandle(Service, Parameter, "uri_handler");
}
