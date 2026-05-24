//! `unregister_uri_handler` — disposes a URI-handler provider handle.
//! Logs the optional bound scheme alongside the handle for traceability.

use serde_json::Value;

use super::Support::Fn::Fn;
use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn Fn(Service:&MountainVinegRPCService, Parameter:&Value) {
	let Scheme = Parameter.get("scheme").and_then(Value::as_str).unwrap_or("");

	dev_log!("provider-register", "[ProviderUnregister] uri_handler scheme={}", Scheme);

	UnregisterByHandle(Service, Parameter, "uri_handler");
}
