#![allow(non_snake_case)]
//! Cocoon → Mountain `unregister_uri_handler` notification.
//! Emitted by `Cocoon/.../WindowNamespace.ts:786` when
//! `vscode.window.registerUriHandler(...).dispose()` fires. Carries the
//! handle the paired `register_uri_handler` stored and (optionally) the
//! scheme bound to it.

use serde_json::Value;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn UnregisterUriHandler(Service:&MountainVinegRPCService, Parameter:&Value) {

	let Handle = Parameter.get("handle").and_then(Value::as_u64).unwrap_or(0) as u32;

	let Scheme = Parameter.get("scheme").and_then(Value::as_str).unwrap_or("");

	if Handle == 0 {

		dev_log!("provider-register", "[ProviderUnregister] uri_handler skip: missing handle");

		return;
	}

	Service
		.RunTime()
		.Environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.UnregisterProvider(Handle);

	dev_log!(
		"provider-register",

		"[ProviderUnregister] uri_handler handle={} scheme={}",

		Handle,

		Scheme
	);
}
