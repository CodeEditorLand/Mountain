#![allow(non_snake_case)]
//! Cocoon → Mountain `webview.setTitle` / `webview.setIconPath` /
//! `webview.setHtml` notifications. Shared atom because the three wire
//! methods map to the same suffix-split pattern; keeping them in one
//! file avoids three near-identical 5-line files while still pinning
//! the handler to a discoverable filename.
//!
//! For per-extension isolation and payload inspection, split this into
//! three atoms (`WebviewSetTitle`, `WebviewSetIconPath`, `WebviewSetHtml`)
//! when the divergence is worth it - the dispatcher would add two more
//! single-line arms.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn WebviewLifecycle(Service:&MountainVinegRPCService, MethodName:&str, Parameter:&Value) {
	// Suffix mapping: stock VS Code wire methods are camelCase
	// (`webview.setHtml`, `webview.setIconPath`), but Sky's canonical
	// channel registry (`Common/Source/IPC/SkyEvent.rs`) standardises
	// kebab-case for set-html (`sky://webview/set-html`) since the
	// `setWebviewHtml` typed-RPC and `Window.rs::SetHtml` request both
	// emit kebab. Translate `setHtml` here so every producer of the
	// "html assignment" wire shape lands on the same Sky channel; other
	// suffixes pass through camelCase (Sky listeners use camel for
	// `setTitle` / `setIconPath`).
	let RawSuffix = &MethodName["webview.".len()..];
	let Suffix = match RawSuffix {
		"setHtml" => "set-html",
		Other => Other,
	};
	let EventName = format!("sky://webview/{}", Suffix);
	if let Err(Error) = Service.ApplicationHandle().emit(&EventName, Parameter) {
		dev_log!("grpc", "warn: [MountainVinegRPCService] {} emit failed: {}", EventName, Error);
	}
}
