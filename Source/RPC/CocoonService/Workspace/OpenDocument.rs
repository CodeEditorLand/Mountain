//! Open a document in the workbench via `sky://editor/openDocument`.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{OpenDocumentRequest, OpenDocumentResponse},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:OpenDocumentRequest,
) -> Result<Response<OpenDocumentResponse>, Status> {
	let URI = Request.uri.as_ref().map(|U| U.value.clone()).unwrap_or_default();

	dev_log!("cocoon", "[CocoonService] open_document: {}", URI);

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://editor/openDocument",
		json!({ "uri": URI, "viewColumn": Request.view_column }),
	);

	Ok(Response::new(OpenDocumentResponse { success:true }))
}
