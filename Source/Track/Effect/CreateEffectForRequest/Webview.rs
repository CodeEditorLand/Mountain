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
			// Per-dispatch entry line - parity with TreeView.rs's
			// `tree-latency` log. Without this we cannot tell from
			// `Mountain.dev.log` whether Cocoon's
			// `MountainClient.sendRequest("webview.registerView", ...)`
			// even reached `DispatchSideCarRequest` - silent gRPC drops
			// look identical to "extension never called the shim".
			dev_log!("ipc", "[WebviewEffect] dispatch-enter method={}", MethodName);
			let Method = MethodName.to_string();
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					let Method = Method.clone();
					Box::pin(async move {
						let Handle = Parameters.get(0).cloned().unwrap_or(Value::Null);
						let Payload = json!({
							"method": Method,
							"handle": Handle,
							"args": Parameters,
						});
						let Suffix = Method.trim_start_matches("$webview:").trim_start_matches("webview.");
						let EventName = format!("sky://webview/{}", Suffix);
						// `LogSkyEmit` wraps `.emit()` and tags every
						// success/failure under `[DEV:SKY-EMIT]`, so
						// the webview channel becomes visible in the
						// SkyEmit histogram alongside SCM and tree-view.
						// The bare `.emit()` was invisible, so a silent
						// listener-side drop in Sky was indistinguishable
						// from "Mountain never received the request".
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
						let resource_uri = Url::parse(resource_uri_str)
							.unwrap_or_else(|_| Url::parse("file:///tmp/test.txt").unwrap());
						let webview_handle =
							Parameters.get(2).and_then(Value::as_str).unwrap_or("webview-123").to_string();
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
