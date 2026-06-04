pub fn Matches(MethodName:&str) -> bool {
	match MethodName {
		debug.dap-response, Debug.Start, Debug.RegisterConfigurationProvider, Debug.Stop => true,
		_ => false,
	}
}

use std::sync::Arc;

use CommonLibrary::{Debug::DebugService::DebugService, Environment::Requires::Requires};
use serde_json::{Value, json};
use tauri::{Emitter, Runtime};
use url::Url;

use crate::Track::Effect::{
	CreateEffectForRequest::Utilities::Params::{i64_at_or, str_at, string_at, string_at_or},
	MappedEffectType::MappedEffect,
};
pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:&Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		// Cocoon's `Debug/Namespace.ts:63` sends `debug.dap-response` as a
		// fire-and-forget notification carrying a DAP response message
		// emitted by an inline-implementation adapter (one that runs
		// inside the extension host, not as a spawned process). Forward
		// to the renderer via `sky://debug/dap-message` so the workbench's
		// RawDebugSession sequencer can correlate it against the pending
		// request by `request_seq`. Payload: `{ sessionId, message }`.
		"debug.dap-response" => {
			crate::effect!(run_time, {
				let session_id = Parameters.get("sessionId").and_then(Value::as_str).unwrap_or("").to_string();

				if session_id.is_empty() {
					return Err("debug.dap-response: missing 'sessionId' field".to_string());
				}

				let message = Parameters.get("message").cloned().unwrap_or(Value::Null);

				let _ = run_time.Environment.ApplicationHandle.emit(
					"sky://debug/dap-message",
					json!({
						"sessionId": session_id,
						"sidecarId": "cocoon-main",
						"message": message,
					}),
				);

				Ok(json!(null))
			})
		},

		"Debug.Start" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn DebugService> = run_time.Environment.Require();

				let folder_uri_str = str_at(&Parameters, 0);

				let folder_uri = if folder_uri_str.is_empty() { None } else { Url::parse(folder_uri_str).ok() };

				let configuration = Parameters.get(1).cloned().unwrap_or_else(|| json!({ "type": "node" }));

				provider
					.StartDebugging(folder_uri, configuration)
					.await
					.map(|session_id| json!(session_id))
					.map_err(|e| e.to_string())
			})
		},

		"Debug.RegisterConfigurationProvider" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn DebugService> = run_time.Environment.Require();

				let debug_type = string_at_or(&Parameters, 0, "node");

				let provider_handle = i64_at_or(&Parameters, 1, 1) as u32;

				let sidecar_id = string_at_or(&Parameters, 2, "cocoon-main");

				provider
					.RegisterDebugConfigurationProvider(debug_type, provider_handle, sidecar_id)
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		"Debug.Stop" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn DebugService> = run_time.Environment.Require();

				let SessionId = string_at(&Parameters, 0);

				provider
					.StopDebugging(SessionId)
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		_ => None,
	}
}
