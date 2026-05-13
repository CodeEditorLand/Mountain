#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{CustomEditor::CustomEditorProvider::CustomEditorProvider, Environment::Requires::Requires};
use serde_json::{Value, json};
use tauri::Runtime;
use url::Url;

use crate::{
	IPC::SkyEmit::LogSkyEmit,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Track::Effect::MappedEffectType::MappedEffect,
	dev_log,
};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"$webview:create"
		| "webview.create"
		| "webview.setHtml"
		| "webview.setOptions"
		| "webview.postMessage"
		| "webview.reveal"
		| "webview.dispose"
		| "webview.registerView"
		| "webview.unregisterView"
		| "webview.registerCustomEditor"
		| "webview.unregisterCustomEditor" => {
			dev_log!("ipc", "[WebviewEffect] dispatch-enter method={}", MethodName);

			let Method = MethodName.to_string();

			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					let Method = Method.clone();

					Box::pin(async move {
						let RawSuffix = Method.trim_start_matches("$webview:").trim_start_matches("webview.");
						let Suffix:&str = match RawSuffix {
							"setHtml" => "set-html",
							"postMessage" => "post-message",
							Other => Other,
						};
						let Payload:Value = if Parameters.is_object() {
							Parameters.clone()
						} else if let Some(First) = Parameters.get(0) {
							if First.is_object() {
								First.clone()
							} else {
								let mut Object = serde_json::Map::new();
								Object.insert("method".to_string(), Value::String(Method.clone()));
								Object.insert("handle".to_string(), First.clone());
								Object.insert("args".to_string(), Parameters.clone());
								if let Some(Second) = Parameters.get(1) {
									let Alias = match Method.as_str() {
										"webview.setHtml" => "html",
										"webview.postMessage" => "message",
										"webview.registerView" | "webview.unregisterView" => "viewId",
										"webview.registerCustomEditor"
										| "webview.unregisterCustomEditor"
										| "webview.create" => "viewType",
										_ => "value",
									};
									Object.insert(Alias.to_string(), Second.clone());
									if Method.as_str() == "webview.create" {
										if let Some(Third) = Parameters.get(2) {
											Object.insert("title".to_string(), Third.clone());
										}
									}
								}
								Value::Object(Object)
							}
						} else {
							json!({
								"method": Method,
								"handle": Parameters.clone(),
							})
						};
						let EventName = format!("sky://webview/{}", Suffix);
						if let Err(Error) = LogSkyEmit(&run_time.Environment.ApplicationHandle, &EventName, &Payload) {
							dev_log!("ipc", "warn: [WebviewEffect] emit {} failed: {}", EventName, Error);
						}
						Ok(json!(null))
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"$resolveCustomEditor" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn CustomEditorProvider> = run_time.Environment.Require();
						let view_type = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let resource_uri_str = Parameters.get(1).and_then(Value::as_str).unwrap_or("");
						// Do not substitute a fallback path for a missing
						// or malformed URI. A silent swap to
						// `file:///tmp/test.txt` would:
						//   - create that file on disk on every bad call,
						//   - return success to Cocoon so the extension never receives an error,
						//   - make the log undiagnosable (every failure shows the same sentinel path).
						// Return Err instead so the grpc layer logs the
						// real caller input.
						if resource_uri_str.is_empty() {
							dev_log!(
								"grpc",
								"warn: [$resolveCustomEditor] empty resource URI view_type={}",
								view_type
							);
							return Err(format!(
								"$resolveCustomEditor: empty resource URI for view_type={}",
								view_type
							));
						}
						let resource_uri = match Url::parse(resource_uri_str) {
							Ok(u) => u,
							Err(parse_err) => {
								dev_log!(
									"grpc",
									"warn: [$resolveCustomEditor] invalid URI uri={} err={} view_type={}",
									resource_uri_str,
									parse_err,
									view_type
								);
								return Err(format!(
									"$resolveCustomEditor: invalid resource URI '{}': {}",
									resource_uri_str, parse_err
								));
							},
						};
						let webview_handle = Parameters.get(2).and_then(Value::as_str).unwrap_or("").to_string();
						if webview_handle.is_empty() {
							dev_log!(
								"grpc",
								"warn: [$resolveCustomEditor] empty webview handle uri={} view_type={}",
								resource_uri_str,
								view_type
							);
							return Err(format!(
								"$resolveCustomEditor: empty webview handle for view_type={} uri={}",
								view_type, resource_uri_str
							));
						}
						provider
							.ResolveCustomEditor(view_type, resource_uri, webview_handle)
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
