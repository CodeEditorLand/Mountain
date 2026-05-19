#![allow(non_snake_case)]
//! Cocoon → Mountain `unregister_debug_configuration_provider` notification.
//! Emitted by `Cocoon/.../DebugNamespace.ts` when an extension disposes a
//! debug configuration provider registered via
//! `vscode.debug.registerDebugConfigurationProvider`. Mirrors
//! `UnregisterDebugAdapter` for the DebugConfiguration slot.

use serde_json::Value;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn UnregisterDebugConfigurationProvider(Service:&MountainVinegRPCService, Parameter:&Value) {
	let Handle = Parameter.get("handle").and_then(Value::as_u64).unwrap_or(0) as u32;

	if Handle == 0 {
		dev_log!(
			"provider-register",
			"[ProviderUnregister] debug_configuration skip: missing handle"
		);

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
		"[ProviderUnregister] debug_configuration handle={}",
		Handle
	);
}
