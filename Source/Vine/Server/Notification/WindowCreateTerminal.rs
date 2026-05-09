#![allow(non_snake_case)]
//! Cocoon → Mountain `window.createTerminal` notification.
//! Fire-and-forget from Cocoon's `vscode.window.createTerminal(...)`
//! shim. Spawns the PTY via the registered `TerminalProvider` so the
//! xterm panel starts receiving data immediately, then emits
//! `sky://terminal/create` with the provider-minted id/pid/name so Sky
//! can correlate the panel with the extension-owned terminal instance.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::Emitter;
use CommonLibrary::{Environment::Requires::Requires, Terminal::TerminalProvider::TerminalProvider};

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn WindowCreateTerminal(Service:&MountainVinegRPCService, Parameter:&Value) {
	let Provider:Arc<dyn TerminalProvider> = Service.RunTime().Environment.Require();

	let Name = Parameter.get("name").and_then(|V| V.as_str()).unwrap_or("terminal").to_string();

	let Options = Parameter.get("options").cloned().unwrap_or_default();

	let Handle = Parameter
		.get("handle")
		.and_then(|V| V.as_str())
		.map(str::to_string)
		.unwrap_or_default();

	let AppHandleForTask = Service.ApplicationHandle().clone();

	let NameForTask = Name.clone();

	tokio::spawn(async move {
		let OptionsPayload = if Options.is_object() {
			let mut Map = Options.as_object().cloned().unwrap_or_default();
			Map.entry("name".to_string()).or_insert_with(|| json!(NameForTask));
			Value::Object(Map)
		} else {
			json!({ "name": NameForTask })
		};
		if let Ok(Created) = Provider.CreateTerminal(OptionsPayload).await {
			if let Err(Error) = AppHandleForTask.emit(
				"sky://terminal/create",
				json!({
					"handle": Handle,
					"id": Created.get("id").cloned().unwrap_or(Value::Null),
					"pid": Created.get("pid").cloned().unwrap_or(Value::Null),
					"name": Created.get("name").cloned().unwrap_or(Value::Null),
				}),
			) {
				dev_log!(
					"grpc",
					"warn: [WindowCreateTerminal] sky://terminal/create emit failed: {}",
					Error
				);
			}
		}
	});
}
