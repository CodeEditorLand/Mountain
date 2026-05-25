//! Cocoon → Mountain `debug.addBreakpoints` / `debug.removeBreakpoints` /
//! `debug.consoleAppend` notifications. Fans on `sky://debug/<suffix>`
//! so the Sky-side debug view picks up breakpoint changes and console
//! output from the extension's `vscode.debug.*` surface.

use serde_json::{Value, json};
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn DebugLifecycle(Service:&MountainVinegRPCService, MethodName:&str, Parameter:&Value) {
	let EventName = format!("sky://debug/{}", &MethodName["debug.".len()..]);

	if let Err(Error) = Service.ApplicationHandle().emit(&EventName, Parameter) {
		dev_log!("grpc", "warn: [MountainVinegRPCService] {} emit failed: {}", EventName, Error);
	}

	// For breakpoint changes specifically, also fan back to Cocoon so
	// `vscode.debug.onDidChangeBreakpoints` subscribers in OTHER
	// extensions observe the change. Without this round-trip, only the
	// extension that called `addBreakpoints`/`removeBreakpoints` knows
	// about its own write - peer extensions miss it. `debug.consoleAppend`
	// doesn't need this; console output isn't an observable surface.
	if MethodName == "debug.addBreakpoints" || MethodName == "debug.removeBreakpoints" {
		let Added:Vec<Value> = if MethodName == "debug.addBreakpoints" {
			Parameter
				.get("breakpoints")
				.and_then(Value::as_array)
				.cloned()
				.unwrap_or_default()
		} else {
			Vec::new()
		};

		let Removed:Vec<Value> = if MethodName == "debug.removeBreakpoints" {
			Parameter
				.get("breakpoints")
				.and_then(Value::as_array)
				.cloned()
				.unwrap_or_default()
		} else {
			Vec::new()
		};

		let _ = crate::Vine::Client::SendNotification::Fn(
			"cocoon-main".to_string(),
			"$onDidChangeBreakpoints".to_string(),
			json!({
				"added": Added,
				"removed": Removed,
				"changed": [],
			}),
		)
		.await;
	}
}
