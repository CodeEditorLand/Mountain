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
	let ScmId = Parameter
		.get("scm_id")
		.or_else(|| Parameter.get("scmId"))
		.and_then(Value::as_str)
		.unwrap_or("");
	if ScmId.is_empty() {
		dev_log!("provider-register", "[ProviderUnregister] scm skip: missing scm_id");
		return;
	}
	let Handle = ScmId
		.as_bytes()
		.iter()
		.fold(0u32, |Acc, B| Acc.wrapping_mul(31).wrapping_add(*B as u32));
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
