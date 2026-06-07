//! Forward an `OpenExternal` request to Sky on
//! `sky://native/openExternal` so the webview can launch the URI in the
//! system browser/handler.

use serde_json::json;

use tauri::Emitter;

use tonic::{Response, Status};

use ::Vine::Generated::{Empty, OpenExternalRequest};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(Service:&CocoonServiceImpl, Request:OpenExternalRequest) -> Result<Response<Empty>, Status> {

	dev_log!("cocoon", "[CocoonService] open_external: {}", Request.uri);

	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://native/openExternal", json!({ "url": Request.uri }));

	Ok(Response::new(Empty {}))
}
