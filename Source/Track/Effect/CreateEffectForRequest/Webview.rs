//! # Webview Effect (CreateEffectForRequest)
//!
//! Effect constructors for webview-related RPC methods from the Cocoon
//! extension host. Maps webview method names (e.g. `webview.create`,
//! `$webview:setHtml`) to Sky event channels (e.g. `sky://webview/create`,
//! `sky://webview/set-html`).
//!
//! ## Methods handled
//!
//! | Method | Sky Event Channel |
//! |---|---|
//! | `$webview:create` / `webview.create` | `sky://webview/create` |
//! | `webview.setHtml` | `sky://webview/set-html` |
//! | `webview.setOptions` | `sky://webview/set-options` |
//! | `webview.postMessage` | `sky://webview/post-message` |
//! | `webview.reveal` | `sky://webview/reveal` |
//! | `webview.Dispose` | `sky://webview/dispose` |
//! | `webview.registerView` | `sky://webview/register-view` |
//! | `webview.unregisterView` | `sky://webview/unregister-view` |
//! | `webview.registerCustomEditor` | `sky://webview/register-custom-editor` |
//! | `webview.unregisterCustomEditor` | `sky://webview/unregister-custom-editor` |
//! | `$resolveCustomEditor` | Direct call to `CustomEditorProvider` trait |

use std::sync::Arc;

use CommonLibrary::{CustomEditor::CustomEditorProvider::CustomEditorProvider, Environment::Requires::Requires};
use serde_json::{Value, json};
use tauri::Runtime;
use url::Url;

use crate::{
	IPC::SkyEmit::Fn,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Track::Effect::{
		CreateEffectForRequest::Utilities::Params::{StrAt, StringAt},
		MappedEffectType::MappedEffect,
	},
	dev_log,
};

pub fn Fn<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"$webview:create"
		| "webview.create"
		| "webview.setHtml"
		| "webview.setOptions"
		| "webview.postMessage"
		| "webview.reveal"
		| "webview.Dispose"
		| "webview.registerView"
		| "webview.unregisterView"
		| "webview.registerCustomEditor"
		| "webview.unregisterCustomEditor" => {
			dev_log!("ipc", "[WebviewEffect] dispatch-enter method={}", MethodName);

			let Method = MethodName.to_string();

			crate::effect!(RunTime, {
				let Method = Method.clone();

				let RawSuffix = Method.trim_start_matches("$webview:").trim_start_matches("webview.");
				let Suffix:&str = match RawSuffix {
					"setHtml" => "set-html",
					"postMessage" => "postMessage",
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
				if let Err(Error) = LogSkyEmit(&RunTime.Environment.ApplicationHandle, &EventName, &Payload) {
					dev_log!("ipc", "warn: [WebviewEffect] emit {} failed: {}", EventName, Error);
				}
				Ok(json!(null))
			})
		},

		"$resolveCustomEditor" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn CustomEditorProvider> = RunTime.Environment.Require();
				let ViewType = StringAt(&Parameters, 0);
				let ResourceUriStr = StrAt(&Parameters, 1);
				// Do not substitute a fallback path for a missing
				// or malformed URI. A silent swap to
				// `file:///tmp/test.txt` would:
				//   - create that file on disk on every bad call,
				//   - return success to Cocoon so the extension never receives an error,
				//   - make the log undiagnosable (every failure shows the same sentinel path).
				// Return Err instead so the grpc layer logs the
				// real caller input.
				if ResourceUriStr.is_empty() {
					dev_log!("grpc", "warn: [$resolveCustomEditor] empty resource URI ViewType={}", ViewType);
					return Err(format!("$resolveCustomEditor: empty resource URI for ViewType={}", ViewType));
				}
				let ResourceUri = match Url::parse(ResourceUriStr) {
					Ok(u) => u,
					Err(parse_err) => {
						dev_log!(
							"grpc",
							"warn: [$resolveCustomEditor] invalid URI uri={} err={} ViewType={}",
							ResourceUriStr,
							parse_err,
							ViewType
						);
						return Err(format!(
							"$resolveCustomEditor: invalid resource URI '{}': {}",
							ResourceUriStr, parse_err
						));
					},
				};
				let WebviewHandle = StringAt(&Parameters, 2);
				if WebviewHandle.is_empty() {
					dev_log!(
						"grpc",
						"warn: [$resolveCustomEditor] empty webview handle uri={} ViewType={}",
						ResourceUriStr,
						ViewType
					);
					return Err(format!(
						"$resolveCustomEditor: empty webview handle for ViewType={} uri={}",
						ViewType, ResourceUriStr
					));
				}
				provider
					.ResolveCustomEditor(ViewType, ResourceUri, WebviewHandle)
					.await
					.map(|_| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		_ => None,
	}
}
