//! Update a webview panel's HTML through the trait so the content is
//! captured in `WebviewStateDTO` and re-servable on reveal/restore.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};
use CommonLibrary::Webview::WebviewProvider::WebviewProvider;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	dev_log,
};

use ::Vine::Generated::{Empty, SetWebviewHtmlRequest};

pub async fn Fn(Service:&CocoonServiceImpl, Request:SetWebviewHtmlRequest) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] set_webview_html: handle={} ({} bytes)",
		Request.handle,
		Request.html.len()
	);

	if let Err(Error) = Service
		.environment
		.SetWebviewHTML(Request.handle.to_string(), Request.html.clone())
		.await
	{
		dev_log!("cocoon", "warn: [CocoonService] set_webview_html trait failed: {}", Error);

		let _ = Service.environment.ApplicationHandle.emit(
			"sky://webview/set-html",
			json!({ "handle": Request.handle, "html": Request.html }),
		);
	}

	Ok(Response::new(Empty {}))
}
