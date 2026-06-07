//! Forward a Cocoon-side configuration change to Sky for workbench
//! settings refresh.

use serde_json::json;

use tauri::Emitter;

use tonic::{Response, Status};

use ::Vine::Generated::{Empty, UpdateConfigurationRequest};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(Service:&CocoonServiceImpl, Request:UpdateConfigurationRequest) -> Result<Response<Empty>, Status> {

	dev_log!(
		"cocoon",

		"[CocoonService] update_configuration: {} changed keys",

		Request.changed_keys.len()
	);

	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://configuration/changed", json!({ "changedKeys": Request.changed_keys }));

	Ok(Response::new(Empty {}))
}
