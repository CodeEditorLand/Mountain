//! Apply a sequence of text edits to a document via
//! `sky://editor/applyEdits`. Each edit carries a `range` (start/end
//! position) plus the replacement `newText`.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	dev_log,
};

use ::Vine::Generated::{ApplyEditRequest, ApplyEditResponse};

pub async fn Fn(Service:&CocoonServiceImpl, Request:ApplyEditRequest) -> Result<Response<ApplyEditResponse>, Status> {
	let URI = Request.uri.as_ref().map(|U| U.value.clone()).unwrap_or_default();

	dev_log!(
		"cocoon",
		"[CocoonService] apply_edit: uri={} edits={}",
		URI,
		Request.edits.len()
	);

	let EditsJSON:Vec<serde_json::Value> = Request
		.edits
		.iter()
		.map(|E| {
			json!({
				"range": {
					"start": E.range.as_ref().and_then(|R| R.start.as_ref()).map(|P| {
						json!({ "line": P.line, "character": P.character })
					}),
					"end": E.range.as_ref().and_then(|R| R.end.as_ref()).map(|P| {
						json!({ "line": P.line, "character": P.character })
					}),
				},
				"newText": E.new_text,
			})
		})
		.collect();

	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://editor/applyEdits", json!({ "uri": URI, "edits": EditsJSON }));

	Ok(Response::new(ApplyEditResponse { success:true }))
}
