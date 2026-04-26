#![allow(non_snake_case)]
//! Cocoon → Mountain `unregister_debug_adapter` notification.
//! Emitted by `Cocoon/.../DebugNamespace.ts:38` when an extension disposes
//! a debug adapter descriptor factory. Mirrors
//! `UnregisterAuthenticationProvider` but for the DebugAdapter slot.

use serde_json::Value;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn UnregisterDebugAdapter(Service:&MountainVinegRPCService, Parameter:&Value) {
	let Handle = Parameter.get("handle").and_then(Value::as_u64).unwrap_or(0) as u32;
	if Handle == 0 {
		dev_log!("provider-register", "[ProviderUnregister] debug_adapter skip: missing handle");
		return;
	}
	Service
		.RunTime()
		.Environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.UnregisterProvider(Handle);
	dev_log!("provider-register", "[ProviderUnregister] debug_adapter handle={}", Handle);
}
