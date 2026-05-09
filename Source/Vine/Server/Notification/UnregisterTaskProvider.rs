#![allow(non_snake_case)]
//! Cocoon → Mountain `unregister_task_provider` notification.
//! Emitted by `Cocoon/.../TasksNamespace.ts:35` when
//! `vscode.tasks.registerTaskProvider(...).dispose()` fires.

use serde_json::Value;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn UnregisterTaskProvider(Service:&MountainVinegRPCService, Parameter:&Value) {

	let Handle = Parameter.get("handle").and_then(Value::as_u64).unwrap_or(0) as u32;

	if Handle == 0 {

		dev_log!("provider-register", "[ProviderUnregister] task skip: missing handle");

		return;
	}

	Service
		.RunTime()
		.Environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.UnregisterProvider(Handle);

	dev_log!("provider-register", "[ProviderUnregister] task handle={}", Handle);
}
