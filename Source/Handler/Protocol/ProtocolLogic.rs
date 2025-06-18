// @module ProtocolLogic
// @description Contains the handler logic for custom URI scheme requests,
// which are registered with Tauri's webview protocol system.

use std::sync::Arc;

use Common::error::CommonError;
use log::{error, info, warn};
use tauri::{
	AppHandle,
	Runtime,
	http::{Request as TauriHttpRequest, Response as TauriHttpResponse, ResponseBuilder},
};
use url::Url;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track, handler::error_utils};

/// The main handler for custom URI scheme requests (e.g., `vscode://file/...`).
///
/// This function is registered with Tauri in `Binary.rs`. It parses the
/// incoming URI, dispatches it to the `track` module to create an appropriate
/// `ActionEffect`, and then executes that effect in the background. It
/// immediately returns an HTTP response to Tauri to acknowledge the request.
///
/// @param request - The raw HTTP request from Tauri's protocol handler.
/// @param app_handle - The Tauri application handle.
/// @returns A `Result` containing the HTTP response to send back to the
/// webview.
pub fn HandleCustomUriSchemeRequest<R:Runtime>(
	request:&TauriHttpRequest,
	app_handle:AppHandle<R>,
) -> Result<TauriHttpResponse, Box<dyn std::error::Error>> {
	info!("[ProtocolLogic] Received custom URI request: {}", request.uri());
	let parsed_url = match Url::parse(request.uri()) {
		Ok(url) => url,
		Err(e) => {
			let error_body = error_utils::RpcErrorString(format!("Failed to parse URI: {}", e), Some("EBADURI"));
			return Ok(ResponseBuilder::new().status(400).body(error_body.into_bytes())?);
		},
	};

	// Delegate to the track module to map the URI to a specific ActionEffect.
	// We'll treat the URL authority + path as the command.
	let command = format!("{}{}", parsed_url.authority(), parsed_url.path());
	let argument = serde_json::to_value(parsed_url.query_pairs().collect::<std::collections::HashMap<_, _>>())?;

	let effect_result = Track::EffectCreation::CreateEffectForFrontendCommand(&app_handle, &command, argument);

	if let Ok(effect) = effect_result {
		let app_handle_clone = app_handle.clone();
		// Spawn the effect on a background task so we can return an immediate HTTP
		// response.
		tauri::async_runtime::spawn(async move {
			let run_time_state:tauri::State<'_, Arc<ApplicationRunTime>> = app_handle_clone.state();
			if let Err(e) = run_time_state.Run(effect).await {
				error!("[ProtocolLogic] Error running effect for URI {}: {:?}", request.uri(), e);
			}
		});
		// Acknowledge that the request was received and is being processed.
		Ok(ResponseBuilder::new().status(200).body(vec![])?)
	} else {
		// If the track couldn't find a corresponding action, return a 404.
		let error_body = effect_result.unwrap_err();
		warn!(
			"[ProtocolLogic] Could not create effect for URI {}: {}",
			request.uri(),
			error_body
		);
		Ok(ResponseBuilder::new().status(404).body(error_body.into_bytes())?)
	}
}
