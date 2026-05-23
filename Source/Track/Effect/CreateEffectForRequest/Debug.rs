use std::sync::Arc;

use CommonLibrary::{Debug::DebugService::DebugService, Environment::Requires::Requires};
use serde_json::{Value, json};
use tauri::Runtime;
use url::Url;

use crate::Track::Effect::{
	CreateEffectForRequest::Utilities::Params::{str_at, string_at, string_at_or},
	MappedEffectType::MappedEffect,
};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
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
				let provider_handle = Parameters.get(1).and_then(Value::as_i64).map(|n| n as u32).unwrap_or(1);
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
