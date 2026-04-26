#![allow(non_snake_case)]
//! Cocoon → Mountain `unregister_file_system_provider` notification.
//! Emitted by `Cocoon/.../WorkspaceNamespace/Providers.ts:78` on
//! `FileSystemProvider` disposal. Scheme-bound: the paired
//! `register_file_system_provider` stores the scheme in the provider
//! selector so filesystem router lookups stop routing to this handle.

use serde_json::Value;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn UnregisterFileSystemProvider(Service:&MountainVinegRPCService, Parameter:&Value) {
	let Handle = Parameter.get("handle").and_then(Value::as_u64).unwrap_or(0) as u32;
	let Scheme = Parameter.get("scheme").and_then(Value::as_str).unwrap_or("");
	if Handle == 0 {
		dev_log!(
			"provider-register",
			"[ProviderUnregister] file_system skip: missing handle (scheme={})",
			Scheme
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
		"[ProviderUnregister] file_system handle={} scheme={}",
		Handle,
		Scheme
	);
}
