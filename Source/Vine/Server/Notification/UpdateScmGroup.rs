#![allow(non_snake_case)]
//! Cocoon → Mountain `update_scm_group` notification.
//! Parallels the typed `RPC/CocoonService/SCM.rs::UpdateScmGroup` gRPC;
//! Cocoon's `ScmNamespace.ts` emits through `SendToMountain(...)` for
//! fire-and-forget resource-state updates. Re-emits on the canonical
//! `sky://scm/updateGroup` channel so the renderer SCM view updates
//! without waiting for a round-trip response.

use serde_json::{Value, json};
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn UpdateScmGroup(Service:&MountainVinegRPCService, Parameter:&Value) {
	let ProviderId = Parameter
		.get("provider_id")
		.or_else(|| Parameter.get("providerId"))
		.and_then(Value::as_str)
		.unwrap_or("");
	let GroupId = Parameter
		.get("group_id")
		.or_else(|| Parameter.get("groupId"))
		.and_then(Value::as_str)
		.unwrap_or("");
	let ResourceStates = Parameter
		.get("resource_states")
		.or_else(|| Parameter.get("resourceStates"))
		.cloned()
		.unwrap_or_else(|| Value::Array(Vec::new()));

	if ProviderId.is_empty() || GroupId.is_empty() {
		dev_log!("grpc", "[ScmGroup] skip: missing provider_id or group_id");
		return;
	}

	let _ = Service.ApplicationHandle().emit(
		"sky://scm/updateGroup",
		json!({
			"providerId": ProviderId,
			"groupId": GroupId,
			"resourceStates": ResourceStates,
		}),
	);
	dev_log!(
		"grpc",
		"[ScmGroup] provider={} group={} resources={}",
		ProviderId,
		GroupId,
		ResourceStates.as_array().map(Vec::len).unwrap_or(0)
	);
}
