//! Save every dirty editor (optionally including untitled) via
//! `sky://editor/saveAll`.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	dev_log,
};

use ::Vine::Generated::{SaveAllRequest, SaveAllResponse};

pub async fn Fn(Service:&CocoonServiceImpl, Request:SaveAllRequest) -> Result<Response<SaveAllResponse>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] save_all: includeUntitled={}",
		Request.include_untitled
	);

	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://editor/saveAll", json!({ "includeUntitled": Request.include_untitled }));

	Ok(Response::new(SaveAllResponse { success:true }))
}
