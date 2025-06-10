use std::sync::Arc;

use Common::{effect::ActionEffect, error::CommonError};
use log::{error, info, warn};
use tauri::{
	AppHandle,
	Runtime,
	http::{Request as TauriHttpRequest, Response as TauriHttpResponse},
};
use url::Url;

/// @module ProtocolLogic
/// @description Contains the handler logic for custom URI scheme requests,
/// which are registered with Tauri's webview protocol system.
use crate::runtime::AppRuntime::AppRuntime;
use crate::{handlers::error_utils, track};

/// The main handler for custom URI scheme requests (e.g., `vscode://file/...`).
///
/// This function is registered with Tauri in `main.rs`. It parses the incoming
/// URI, dispatches it to the `track` module to create an appropriate
/// `ActionEffect`, and then executes that effect in the background. It
/// immediately returns an HTTP response to Tauri to acknowledge the request.
///
/// @param Request - The raw HTTP request from Tauri's protocol handler.
/// @param AppHandle - The Tauri application handle.
/// @returns A `Result` containing the HTTP response to send back to the
/// webview.
pub fn HandleCustomUriSchemeRequest<R:Runtime>(
	Request:&TauriHttpRequest,
	AppHandle:AppHandle<R>,
) -> Result<TauriHttpResponse, Box<dyn std::error::Error>> {
	info!("[ProtocolLogic] Received custom URI request: {}", Request.uri());
	let ParsedUrl = match Url::parse(Request.uri()) {
		Ok(Url) => Url,
		Err(e) => {
			let ErrorBody = error_utils::RpcErrorString(format!("Failed to parse URI: {}", e), Some("EBADURI"));
			return Ok(TauriHttpResponse::builder().status(400).body(ErrorBody.into_bytes())?);
		},
	};

	// Delegate to the track module to map the URI to a specific ActionEffect.
	let EffectResult:Result<ActionEffect<Arc<AppRuntime>, CommonError, serde_json::Value>, String> =
		track::CreateEffectForUriProtocol(&ParsedUrl);

	if let Ok(Effect) = EffectResult {
		let AppHandleClone = AppHandle.clone();
		// Spawn the effect on a background task so we can return an immediate HTTP
		// response.
		tauri::async_runtime::spawn(async move {
			let RuntimeState:tauri::State<'_, Arc<AppRuntime>> = AppHandleClone.state();
			if let Err(e) = RuntimeState.Run(Effect).await {
				error!("[ProtocolLogic] Error running effect for URI {}: {:?}", Request.uri(), e);
			}
		});
		// Acknowledge that the request was received and is being processed.
		Ok(TauriHttpResponse::builder().status(200).body(vec![])?)
	} else {
		// If the track couldn't find a corresponding action, return a 404.
		let ErrorBody = EffectResult.unwrap_err();
		warn!(
			"[ProtocolLogic] Could not create effect for URI {}: {}",
			Request.uri(),
			ErrorBody
		);
		Ok(TauriHttpResponse::builder().status(404).body(ErrorBody.into_bytes())?)
	}
}
