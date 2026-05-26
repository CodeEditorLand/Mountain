//! Cocoon → Mountain `update_scm_group` notification.
//!
//! Parallels the typed `RPC/CocoonService/SCM.rs::UpdateScmGroup` gRPC;
//! Cocoon's `ScmNamespace.ts` emits through `SendToMountain(...)` for
//! fire-and-forget resource-state updates. Re-emits on the canonical
//! `sky://scm/updateGroup` channel so the renderer SCM view updates
//! without waiting for a round-trip response.
//!
//! Wire shape (from `ScmNamespace.ts:108`):
//!
//! ```ignore
//! { scm_handle: u32, group_handle: "<scm_handle>/<group_id>", resource_states: [...] }
//! ```
//!
//! Earlier revisions of this atom read `provider_id`/`group_id` and
//! silently dropped every update because Cocoon never sends those keys
//! - the resulting `[ScmGroup] skip: missing provider_id or group_id`
//! line was the only signal the SCM viewlet was being starved. The
//! current decoder reads the canonical handle pair, splits the
//! `<handle>/<groupId>` form for the renderer payload, and falls back
//! to the legacy `provider_id`/`group_id` keys for any stale caller
//! that hasn't migrated yet.

use std::collections::HashMap;

use CommonLibrary::SourceControlManagement::DTO::SourceControlManagementResourceDTO::SourceControlManagementResourceDTO;
use serde_json::{Value, json};
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn UpdateScmGroup(Service:&MountainVinegRPCService, Parameter:&Value) {
	// Producer (Cocoon `ScmNamespace.ts`) emits camelCase keys post-audit.
	// snake_case probes retained as transitional fallback for one rebuild.
	let ScmHandle = Parameter
		.get("scmHandle")
		.or_else(|| Parameter.get("scm_handle"))
		.and_then(Value::as_u64)
		.map(|H| H as u32);

	let GroupHandle = Parameter
		.get("groupHandle")
		.or_else(|| Parameter.get("group_handle"))
		.and_then(Value::as_str)
		.unwrap_or("")
		.to_string();

	// Legacy fallbacks: pre-2026-04 Cocoon revisions used flat
	// `provider_id`/`group_id`. Keep parsing them so a downgrade of
	// just one side does not silently drop traffic.
	let LegacyProviderId = Parameter
		.get("providerId")
		.or_else(|| Parameter.get("provider_id"))
		.and_then(Value::as_str)
		.unwrap_or("")
		.to_string();

	let LegacyGroupId = Parameter
		.get("groupId")
		.or_else(|| Parameter.get("group_id"))
		.and_then(Value::as_str)
		.unwrap_or("")
		.to_string();

	let ResourceStates = Parameter
		.get("resourceStates")
		.or_else(|| Parameter.get("resource_states"))
		.cloned()
		.unwrap_or_else(|| Value::Array(Vec::new()));

	// `group_handle` is `"<scm_handle>/<group_id>"` per ScmNamespace.ts:77.
	// Split for the renderer payload so the existing
	// `cel:scm:updateGroup` listeners (which expect a flat `groupId`)
	// keep working without forcing them to re-parse.
	let (HandleFromString, GroupIdFromHandle) = match GroupHandle.split_once('/') {
		Some((H, G)) => (H.parse::<u32>().ok(), G.to_string()),

		None => (None, String::new()),
	};

	let ResolvedScmHandle = ScmHandle.or(HandleFromString);

	let ResolvedGroupId = if !GroupIdFromHandle.is_empty() {
		GroupIdFromHandle
	} else if !LegacyGroupId.is_empty() {
		LegacyGroupId
	} else {
		String::new()
	};

	if ResolvedScmHandle.is_none() && LegacyProviderId.is_empty() {
		dev_log!(
			"grpc",
			"[ScmGroup] skip: missing scm_handle / provider_id (group_handle={:?} legacy_group={:?})",
			GroupHandle,
			ResolvedGroupId
		);

		return;
	}

	if ResolvedGroupId.is_empty() {
		dev_log!(
			"grpc",
			"[ScmGroup] skip: missing group_id (scm_handle={:?} group_handle={:?})",
			ResolvedScmHandle,
			GroupHandle
		);

		return;
	}

	// Persist the resource snapshot into MarkerState so the boot-time replay
	// (`Sky/ReplayEvents.rs`) can re-emit it for any listener that subscribed
	// late. Without this, every UpdateScmGroup arrival only fired its Tauri
	// event and the in-process Markers map stayed empty - SCM panels that
	// rendered after the live event missed (and the replay loop with it).
	if let Some(ScmHandleVal) = ResolvedScmHandle
		&& let Ok(mut Resources) = Service
			.RunTime()
			.Environment
			.ApplicationState
			.Feature
			.Markers
			.SourceControlManagementResources
			.lock()
	{
		let GroupsForProvider = Resources.entry(ScmHandleVal).or_insert_with(HashMap::new);

		let mut DtoList:Vec<SourceControlManagementResourceDTO> = Vec::new();

		if let Some(Array) = ResourceStates.as_array() {
			for Raw in Array {
				let ResourceUri = Raw
					.get("resourceUri")
					.or_else(|| Raw.get("sourceUri"))
					.or_else(|| Raw.get("uri"))
					.cloned()
					.unwrap_or(Value::Null);

				if ResourceUri.is_null() {
					continue;
				}

				let Decorations = Raw.get("decorations").cloned().unwrap_or(Value::Object(serde_json::Map::new()));

				DtoList.push(SourceControlManagementResourceDTO {
					ProviderHandle:ScmHandleVal,
					GroupIdentifier:ResolvedGroupId.clone(),
					ResourceURI:ResourceUri,
					Decorations,
				});
			}
		}

		GroupsForProvider.insert(ResolvedGroupId.clone(), DtoList);
	}

	let _ = Service.ApplicationHandle().emit(
		"sky://scm/updateGroup",
		json!({
			"scmHandle": ResolvedScmHandle,
			"providerId": &LegacyProviderId,
			"groupHandle": &GroupHandle,
			"groupId": &ResolvedGroupId,
			"resourceStates": ResourceStates,
		}),
	);

	dev_log!(
		"grpc",
		"[ScmGroup] scm_handle={:?} group={} resources={}",
		ResolvedScmHandle,
		ResolvedGroupId,
		ResourceStates.as_array().map(Vec::len).unwrap_or(0)
	);
}
