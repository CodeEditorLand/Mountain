
//! Update a registered SCM provider's resource-state group via the trait
//! (which mutates state and emits the deduplicated UI event). Falls back
//! to a direct Sky emit if the trait wiring is unavailable.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};
use CommonLibrary::SourceControlManagement::SourceControlManagementProvider::SourceControlManagementProvider;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, UpdateScmGroupRequest},
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:UpdateScmGroupRequest) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] update_scm_group: provider={} group={}",
		Request.provider_id,
		Request.group_id
	);

	let ResourceStates:Vec<serde_json::Value> = Request
		.resource_states
		.iter()
		.map(|RS| {
			json!({
				"uri": RS.uri.as_ref().map(|U| U.value.as_str()).unwrap_or(""),
				"decorations": RS.decorations,
			})
		})
		.collect();

	let ProviderHandle = Request
		.provider_id
		.as_bytes()
		.iter()
		.fold(0u32, |Acc, B| Acc.wrapping_mul(31).wrapping_add(*B as u32));

	let GroupData = json!({
		"groupId": Request.group_id,
		"label": Request.group_id,
		"resourceStates": ResourceStates,
	});

	if let Err(Error) = Service.environment.UpdateSourceControlGroup(ProviderHandle, GroupData).await {
		dev_log!(
			"cocoon",
			"warn: [CocoonService] UpdateSourceControlGroup trait failed ({}); falling back to Sky emit",
			Error
		);

		let _ = Service.environment.ApplicationHandle.emit(
			"sky://scm/updateGroup",
			json!({
				"providerId": Request.provider_id,
				"groupId": Request.group_id,
				"resourceStates": ResourceStates,
			}),
		);
	}

	Ok(Response::new(Empty {}))
}
