//! Forward resource-state updates from the Vine gRPC notification to Sky.
//! Group label/metadata is already stored from register_scm_resource_group.

use serde_json::json;

use tauri::Emitter;

use tonic::{Response, Status};

use ::Vine::Generated::{Empty, UpdateScmGroupRequest};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

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

	// update_scm_group carries resource states, not group metadata.
	// Emit directly to Sky so the SCM panel shows changed files.
	// scmId + providerId both set so Sky's ResolveScmShim finds the shim.
	let _ = Service.environment.ApplicationHandle.emit(
		"sky://scm/updateGroup",

		json!({
			"scmId": Request.provider_id,
			"providerId": Request.provider_id,
			"groupId": Request.group_id,
			"resourceStates": ResourceStates,
		}),
	);

	Ok(Response::new(Empty {}))
}
