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
						let RawSuffix = Method.trim_start_matches("$webview:").trim_start_matches("webview.");
						// SkyBridge's webview listener registry uses
						// kebab-case for `set-html` and `post-message`
						// (canonical channel name in
						// `Common/Source/IPC/SkyEvent.rs::WebviewSetHTML`),
						// but the Cocoon-side wire method uses camelCase
						// (`webview.setHtml`, `webview.postMessage`).
						// Without this translation, Roo / claude-vscode /
						// any extension that calls
						// `webview.html = "<html>"` emitted on
						// `sky://webview/setHtml` and Sky's listener
						// (registered on `set-html`) silently dropped
						// every payload - the panel rendered the chrome
						// but the iframe stayed blank. Same fix the
						// `Vine/Server/Notification/WebviewLifecycle.rs`
						// path already applies; centralise here so both
						// emit paths land on the same canonical channel.
						// `postMessage` Sky has BOTH listeners (camel +
						// kebab) so either works there, but normalise to
						// kebab for consistency.
						let Suffix:&str = match RawSuffix {
							"setHtml" => "set-html",
							"postMessage" => "post-message",
							Other => Other,
						};
						// Payload-shape canonicalisation. Cocoon's
						// `WindowNamespace.ts` calls
						// `Context.SendToMountain("webview.setHtml",
						// { handle, viewId, html })` for webview-views
						// (Roo, claude-vscode sidebars) and
						// `MountainClient.sendRequest("webview.setHtml",
						// [Handle, Value])` for webview-panels (legacy).
						// SkyBridge's `sky://webview/set-html` listener
						// reads `Payload.viewId` and `Payload.html`
						// directly, so we always emit the named-key
						// shape. Three observed wire shapes:
						//   1. `Parameters` IS the object directly (modern named-arg sendRequest).
						//   2. `Parameters` is `[ <object> ]` (array wrap).
						//   3. `Parameters` is `[ Handle, Value ]` (positional, panel path).
						// The previous code wrapped payloads in
						// `{ method, handle, args }` which made
						// `Payload.viewId === undefined`; the listener
						// returned early and the iframe stayed blank.
						// Add a `name`/`viewId` fallback step too so
						// case-1 payloads that only carry `handle` still
						// reach Sky's registry lookup (Sky maintains a
						// handle→view map under
						// `__CEL_WEBVIEW_VIEWS_BY_HANDLE__`).
						let Payload:Value = if Parameters.is_object() {
							// Case 1: object directly. Pass through.
							Parameters.clone()
						} else if let Some(First) = Parameters.get(0) {
							if First.is_object() {
								// Case 2: array-wrapped object. Unwrap.
								First.clone()
							} else {
								// Case 3: positional `[Handle, Second?, ...]`.
								//
								// SkyBridge's listeners are split between
								// two reading idioms:
								//   - Named keys: `Payload.viewId`, `Payload.html`, `Payload.message`
								//     (set-html, post-message, register/unregisterView).
								//   - Positional `Payload.args[N]`: create (`[Handle, ViewType, Title,
								//     ShowOptions, Options]`), registerCustomEditor (`[Handle, ViewType,
								//     Options]`), setOptions, reveal, dispose.
								//
								// Always preserve the original args array AND
								// add the per-method named alias so a
								// listener using either idiom finds its data.
								// Without the alias every
								// `registerWebviewViewProvider` call from
								// Cocoon emitted a payload whose
								// `viewId === undefined`, the listener
								// early-returned, and the workbench's
								// `IWebviewViewService` registry stayed empty
								// - every extension sidebar (Roo, Codex,
								// gitlens, claude-code, dashboard) painted
								// only the `pre/index.html` chrome with no
								// host content.
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
									// `webview.create`'s args slot 2 is the
									// extension-supplied panel title.
									// SkyBridge's `first-create` diagnostic
									// surfaces it under `Payload.title`;
									// mirror that here so the named-key
									// idiom doesn't lag the positional one.
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
