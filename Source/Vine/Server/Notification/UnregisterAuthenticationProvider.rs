#![allow(non_snake_case)]
//! Cocoon → Mountain `unregister_authentication_provider` notification.
//! Emitted by `Cocoon/.../AuthenticationNamespace.ts:43` when an extension
//! disposes an authentication provider handle. Removes the matching
//! `ProviderRegistrationDTO` so stale provider state doesn't pin memory
//! or shadow a later re-register.

use serde_json::Value;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn UnregisterAuthenticationProvider(Service:&MountainVinegRPCService, Parameter:&Value) {

	let Handle = Parameter.get("handle").and_then(Value::as_u64).unwrap_or(0) as u32;

	if Handle == 0 {

		dev_log!("provider-register", "[ProviderUnregister] authentication skip: missing handle");

		return;
	}

	Service
		.RunTime()
		.Environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.UnregisterProvider(Handle);

	dev_log!("provider-register", "[ProviderUnregister] authentication handle={}", Handle);
}
