
use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{Debug::DebugService::DebugService, Environment::Requires::Requires};
use serde_json::{Value, json};
use tauri::Runtime;
use url::Url;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"Debug.Start" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn DebugService> = run_time.Environment.Require();
						let folder_uri_str = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						let folder_uri = if folder_uri_str.is_empty() { None } else { Url::parse(folder_uri_str).ok() };
						let configuration = Parameters.get(1).cloned().unwrap_or_else(|| json!({ "type": "node" }));
						provider
							.StartDebugging(folder_uri, configuration)
							.await
							.map(|session_id| json!(session_id))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"Debug.RegisterConfigurationProvider" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn DebugService> = run_time.Environment.Require();
						let debug_type = Parameters.get(0).and_then(Value::as_str).unwrap_or("node").to_string();
						let provider_handle = Parameters.get(1).and_then(Value::as_i64).map(|n| n as u32).unwrap_or(1);
						let sidecar_id = Parameters.get(2).and_then(Value::as_str).unwrap_or("cocoon-main").to_string();
						provider
							.RegisterDebugConfigurationProvider(debug_type, provider_handle, sidecar_id)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"Debug.Stop" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn DebugService> = run_time.Environment.Require();
						let SessionId = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						provider
							.StopDebugging(SessionId)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
