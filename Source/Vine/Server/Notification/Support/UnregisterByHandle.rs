//! Shared helper for provider-unregistration notification atoms.
//!
//! Every `unregister_*_provider` wire method does the same three steps:
//! read the `handle` u32 from the parameter, call
//! `ProviderRegistration::UnregisterProvider`, and emit a tagged dev-log
//! line. This function centralises that triple so each atom shrinks to a
//! single delegation call.
//!
//! `TypeName` is the provider kind string used in the log line
//! (e.g. `"authentication"`, `"debug_adapter"`, `"task"`).

use serde_json::Value;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub fn UnregisterByHandle(Service:&MountainVinegRPCService, Parameter:&Value, TypeName:&str) {
	let Handle = Parameter.get("handle").and_then(Value::as_u64).unwrap_or(0) as u32;

	if Handle == 0 {
		dev_log!("provider-register", "[ProviderUnregister] {} skip: missing handle", TypeName);

		return;
	}

	Service
		.RunTime()
		.Environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.UnregisterProvider(Handle);

	dev_log!("provider-register", "[ProviderUnregister] {} handle={}", TypeName, Handle);
}
