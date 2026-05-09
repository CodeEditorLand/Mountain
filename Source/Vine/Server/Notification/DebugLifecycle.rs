#![allow(non_snake_case)]
//! Cocoon → Mountain `debug.addBreakpoints` / `debug.removeBreakpoints` /
//! `debug.consoleAppend` notifications. Fans on `sky://debug/<suffix>`
//! so the Sky-side debug view picks up breakpoint changes and console
//! output from the extension's `vscode.debug.*` surface.

use serde_json::Value;

use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn DebugLifecycle(Service:&MountainVinegRPCService, MethodName:&str, Parameter:&Value) {

	let EventName = format!("sky://debug/{}", &MethodName["debug.".len()..]);

	if let Err(Error) = Service.ApplicationHandle().emit(&EventName, Parameter) {

		dev_log!("grpc", "warn: [MountainVinegRPCService] {} emit failed: {}", EventName, Error);
	}
}
