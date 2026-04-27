#![allow(non_snake_case)]
//! Cocoon → Mountain `unregister_scm_provider` notification.
//! Emitted by `Cocoon/.../ScmNamespace.ts:82` when
//! `vscode.scm.createSourceControl(...).dispose()` fires. The paired
//! `RegisterScmProvider` typed gRPC (`RPC/CocoonService/SCM.rs`) derives
//! the handle as a DJB-style hash of the `scmId`; we recompute the same
//! hash here so unregister cleans up the exact entry `RegisterScmProvider`
//! stored without needing Cocoon to hand the u32 back over the wire.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn UnregisterScmProvider(Service:&MountainVinegRPCService, Parameter:&Value) {
	// Cocoon's `ScmNamespace.ts:dispose` sends only `{ handle }` (number).
	// `scmId` (camelCase) and `scm_id` (legacy snake_case) are also
	// probed for callers that send the string identifier instead.
	let ScmId = Parameter
		.get("scmId")
		.or_else(|| Parameter.get("scm_id"))
		.and_then(Value::as_str)
		.unwrap_or("")
		.to_string();
	let DirectHandle = Parameter.get("handle").and_then(Value::as_u64).map(|H| H as u32);
	if ScmId.is_empty() && DirectHandle.is_none() {
		dev_log!(
			"provider-register",
			"[ProviderUnregister] scm skip: missing handle / scmId"
		);
		return;
	}
	let Handle = DirectHandle.unwrap_or_else(|| {
		ScmId
			.as_bytes()
			.iter()
			.fold(0u32, |Acc, B| Acc.wrapping_mul(31).wrapping_add(*B as u32))
	});
	Service
		.RunTime()
		.Environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.UnregisterProvider(Handle);
	let _ = Service
		.ApplicationHandle()
		.emit("sky://scm/unregister", serde_json::json!({ "scmId": ScmId }));
	dev_log!(
		"provider-register",
		"[ProviderUnregister] scm scm_id={} handle={}",
		ScmId,
		Handle
	);
}
