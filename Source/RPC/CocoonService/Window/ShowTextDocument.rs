//! Open a document in the workbench. Maps to `sky://editor/openDocument`
//! (same channel as `Workspace::OpenDocument::Fn`; this is the
//! window-namespace alias).

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	dev_log,
};

use ::Vine::Generated::{ShowTextDocumentRequest, ShowTextDocumentResponse};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ShowTextDocumentRequest,
) -> Result<Response<ShowTextDocumentResponse>, Status> {
	let URI = Request.uri.as_ref().map(|U| U.value.clone()).unwrap_or_default();

	dev_log!("cocoon", "[CocoonService] show_text_document: {}", URI);

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://editor/openDocument",
		json!({ "uri": URI, "viewColumn": Request.view_column }),
	);

	Ok(Response::new(ShowTextDocumentResponse { success:true }))
}
