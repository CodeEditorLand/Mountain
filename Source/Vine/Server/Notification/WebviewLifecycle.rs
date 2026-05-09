#![allow(non_snake_case)]
//! Cocoon → Mountain `webview.setTitle` / `webview.setIconPath` /
//! `webview.setHtml` / `webview.postMessage` / `webview.updateView` /
//! `webview.viewState` / `webview.dispose` notifications. Shared atom
//! because the methods all map to the same suffix-split pattern; keeping
//! them in one file avoids near-identical 5-line files while still
//! pinning the handler to a discoverable filename.
//!
//! Wire-shape canonicalisation MIRRORS `Track/Effect/CreateEffectForRequest/
//! Webview.rs` so notification-path payloads land on the same named-key
//! shapes as the request path. SkyBridge's listeners read `Payload.viewId`,
//! `Payload.html`, `Payload.message` etc. directly; without this Cocoon's
//! legacy positional `[Handle, Value]` notifications would emit a payload
//! whose only top-level keys are `0`/`1` (array indices), the listener
//! would early-return on the missing named keys, and the iframe would
//! stay blank even when the request path canonicalised correctly.
//!
//! For per-extension isolation and payload inspection, split this into
//! per-method atoms (`WebviewSetTitle`, `WebviewSetIconPath`, etc.) when
//! the divergence is worth it.

use serde_json::{Map, Value, json};
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn WebviewLifecycle(Service:&MountainVinegRPCService, MethodName:&str, Parameter:&Value) {
	// Suffix mapping: stock VS Code wire methods are camelCase
	// (`webview.setHtml`, `webview.setIconPath`), but Sky's canonical
	// channel registry (`Common/Source/IPC/SkyEvent.rs`) standardises
	// kebab-case for set-html (`sky://webview/set-html`) since the
	// `setWebviewHtml` typed-RPC and `Window.rs::SetHtml` request both
	// emit kebab. Translate `setHtml` and `postMessage` here so every
	// producer of those wire shapes lands on the same Sky channel; other
	// suffixes pass through camelCase (Sky listeners use camel for
	// `setTitle` / `setIconPath`).
	let RawSuffix = &MethodName["webview.".len()..];

	let Suffix = match RawSuffix {
		"setHtml" => "set-html",

		"postMessage" => "post-message",

		Other => Other,
	};

	let EventName = format!("sky://webview/{}", Suffix);

	// Canonicalise payload shapes to the same named-key form the request
	// path produces. Three observed cases (matching `Webview.rs`):
	//   1. Object: pass through (Cocoon's modern named-key
	//      `SendToMountain("webview.setHtml", { handle, viewId, html })`).
	//   2. Array `[<obj>]`: unwrap.
	//   3. Array `[Handle, Second?, ...]`: positional - preserve the original args
	//      slot AND project to the per-method named alias so listeners that read
	//      `Payload.html` / `Payload.viewId` / `Payload.message` etc. stay
	//      decoupled from the wire shape.
	let CanonicalPayload:Value = if Parameter.is_object() {
		Parameter.clone()
	} else if let Some(First) = Parameter.get(0) {
		if First.is_object() {
			First.clone()
		} else {
			let mut Object = Map::new();

			Object.insert("method".to_string(), Value::String(MethodName.to_string()));

			Object.insert("handle".to_string(), First.clone());

			Object.insert("args".to_string(), Parameter.clone());

			if let Some(Second) = Parameter.get(1) {
				let Alias = match MethodName {
					"webview.setHtml" => "html",

					"webview.postMessage" => "message",

					"webview.registerView" | "webview.unregisterView" => "viewId",

					"webview.registerCustomEditor" | "webview.unregisterCustomEditor" | "webview.create" => "viewType",

					_ => "value",
				};

				Object.insert(Alias.to_string(), Second.clone());

				if MethodName == "webview.create" {
					if let Some(Third) = Parameter.get(2) {
						Object.insert("title".to_string(), Third.clone());
					}
				}
			}

			Value::Object(Object)
		}
	} else {
		json!({
			"method": MethodName,
			"handle": Parameter.clone(),
		})
	};

	if let Err(Error) = Service.ApplicationHandle().emit(&EventName, &CanonicalPayload) {
		dev_log!("grpc", "warn: [MountainVinegRPCService] {} emit failed: {}", EventName, Error);
	}
}
