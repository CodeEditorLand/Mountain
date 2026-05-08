#![allow(non_snake_case)]
//! Cocoon → Mountain `register_scm_resource_group` notification.
//!
//! Pairs with `RegisterScmProvider`: an SCM provider creates one or more
//! resource groups (Git's "Changes", "Staged Changes", "Merge Changes").
//! Cocoon emits this from `ScmNamespace.ts:42` whenever
//! `sourceControl.createResourceGroup(id, label)` is called by an
//! extension. Wire payload:
//!
//! ```ignore
//! { scm_handle, group_handle, group_id, label }
//! ```
//!
//! The renderer SCM view subscribes to `sky://scm/registerGroup` to
//! materialise the group header row; the typed
//! `SourceControlManagementProvider::UpdateSourceControlGroup` trait
//! seeds the group with an empty `resourceStates` list so the
//! state-tracking path is also primed for the first `update_scm_group`
//! that follows.

use serde_json::{Value, json};
use tauri::Emitter;
use CommonLibrary::SourceControlManagement::SourceControlManagementProvider::SourceControlManagementProvider;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn RegisterScmResourceGroup(Service:&MountainVinegRPCService, Parameter:&Value) {
	// Producer (Cocoon `ScmNamespace.ts`) emits camelCase keys post-audit.
	let ScmHandle = Parameter
		.get("scmHandle")
		.or_else(|| Parameter.get("scm_handle"))
		.and_then(Value::as_u64)
		.unwrap_or(0) as u32;

	let GroupHandleStr = Parameter
		.get("groupHandle")
		.or_else(|| Parameter.get("group_handle"))
		.and_then(Value::as_str)
		.unwrap_or("")
		.to_string();

	let GroupId = Parameter
		.get("groupId")
		.or_else(|| Parameter.get("group_id"))
		.and_then(Value::as_str)
		.unwrap_or("")
		.to_string();

	let Label = Parameter.get("label").and_then(Value::as_str).unwrap_or(&GroupId).to_string();

	if GroupId.is_empty() {
		dev_log!("provider-register", "[ProviderRegister] scm-group skip: missing group_id");

		return;
	}

	// Seed the group through the trait so subsequent `update_scm_group`
	// calls can locate it. UpdateSourceControlGroup is an upsert - it
	// creates the entry on first call - so this primes state without
	// requiring a separate "create-group" trait method. Field names
	// must match `SourceControlGroupUpdateDTO`'s camelCase wire shape
	// (post-DTO-audit): `providerHandle`, `groupId`, `label`.
	let GroupData = json!({
		"providerHandle": ScmHandle,
		"groupId": &GroupId,
		"label": &Label,
		"resourceStates": [],
	});

	if let Err(Error) = Service
		.RunTime()
		.Environment
		.UpdateSourceControlGroup(ScmHandle, GroupData)
		.await
	{
		dev_log!(
			"grpc",
			"warn: [Scm] UpdateSourceControlGroup (seed) failed scm={} group={}: {}",
			ScmHandle,
			GroupId,
			Error
		);
	}

	let _ = Service.ApplicationHandle().emit(
		"sky://scm/registerGroup",
		json!({
			"scmHandle": ScmHandle,
			"groupHandle": &GroupHandleStr,
			"groupId": &GroupId,
			"label": &Label,
		}),
	);

	dev_log!(
		"grpc",
		"[Scm] register group scm_handle={} group_id={} label={}",
		ScmHandle,
		GroupId,
		Label
	);
}
