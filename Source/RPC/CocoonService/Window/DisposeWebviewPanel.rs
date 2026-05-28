//! Dispose a webview panel. The Sky listener at `SkyBridge.ts:2344`
//! destructures `{ panelId }`; the older sibling emitter at
//! `RPC/CocoonService/mod.rs:1235` already uses `panelId` - keep this
//! site aligned so a `dispose` from either path lands in the same DOM
//! `cel:webview:dispose` CustomEvent.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	dev_log,
};

use ::Vine::Generated::{DisposeWebviewPanelRequest, Empty};

pub async fn Fn(Service:&CocoonServiceImpl, Request:DisposeWebviewPanelRequest) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] dispose_webview_panel: handle={}", Request.handle);

	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://webview/dispose", json!({ "panelId": Request.handle }));

	Ok(Response::new(Empty {}))
}
