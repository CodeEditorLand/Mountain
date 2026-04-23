#![allow(non_snake_case)]
//! Cocoon → Mountain `window.createTextEditorDecorationType` /
//! `window.disposeTextEditorDecorationType` notifications. Forwards the
//! payload on `sky://decoration/<suffix>`; Sky's editor renderer owns
//! the Monaco-side decoration lifecycle so Mountain is a pure relay.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn DecorationTypeLifecycle(Service:&MountainVinegRPCService, MethodName:&str, Parameter:&Value) {
	let EventName = format!("sky://decoration/{}", &MethodName["window.".len()..]);
	if let Err(Error) = Service.ApplicationHandle().emit(&EventName, Parameter) {
		dev_log!("grpc", "warn: [MountainVinegRPCService] {} emit failed: {}", EventName, Error);
	}
}
