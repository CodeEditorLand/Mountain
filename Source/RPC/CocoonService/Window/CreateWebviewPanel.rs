#![allow(non_snake_case)]

//! Register a webview panel through the `WebviewProvider` trait so the
//! panel is tracked in `ApplicationState::WebviewState`. Without trait
//! registration `DisposeWebviewPanel` later fails with "unknown handle"
//! and webviews leak DOM. Falls back to a millisecond pseudo-handle and
//! a direct Sky emit on trait failure.

use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

use tauri::Emitter;

use tonic::{Response, Status};

use CommonLibrary::Webview::WebviewProvider::WebviewProvider;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{CreateWebviewPanelRequest, CreateWebviewPanelResponse},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:CreateWebviewPanelRequest,
) -> Result<Response<CreateWebviewPanelResponse>, Status> {

	dev_log!(
		"cocoon",

		"[CocoonService] create_webview_panel: view_type={} title={}",

		Request.view_type,

		Request.title
	);

	let Handle = match Service
		.environment
		.CreateWebviewPanel(
			json!({}),

			Request.view_type.clone(),

			Request.title.clone(),

			json!({ "viewColumn": Request.view_column, "preserveFocus": Request.preserve_focus }),

			json!({}),

			json!({}),
		)
		.await
	{

		Ok(H) => H,

		Err(Error) => {

			dev_log!("cocoon", "warn: [CocoonService] create_webview_panel trait failed: {}", Error);

			let Fallback = SystemTime::now()
				.duration_since(UNIX_EPOCH)
				.map(|D| D.as_millis() as u32)
				.unwrap_or(0);

			let _ = Service.environment.ApplicationHandle.emit(
				"sky://webview/create",

				json!({
					"handle": Fallback,
					"viewType": Request.view_type,
					"title": Request.title,
					"viewColumn": Request.view_column,
					"preserveFocus": Request.preserve_focus,
					"iconPath": Request.icon_path,
				}),
			);

			return Ok(Response::new(CreateWebviewPanelResponse { handle:Fallback }));
		},
	};

	let HandleU32 = Handle
		.parse::<u32>()
		.unwrap_or_else(|_| Handle.chars().map(|C| C as u32).fold(0u32, |Acc, Char| Acc.wrapping_add(Char)));

	Ok(Response::new(CreateWebviewPanelResponse { handle:HandleU32 }))
}
