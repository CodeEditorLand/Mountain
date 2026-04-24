#![allow(non_snake_case)]
//! Cocoon → Mountain `unregisterCommand` notification.
//! Paired with `registerCommand`; removes the proxied
//! `CommandHandler` so subsequent `commands.executeCommand` no longer
//! routes back to the extension.

use serde_json::Value;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn UnregisterCommand(Service:&MountainVinegRPCService, Parameter:&Value) {
	let CommandId = Parameter.get("commandId").and_then(Value::as_str).unwrap_or("");
	if CommandId.is_empty() {
		return;
	}
	if let Ok(mut Registry) = Service
		.RunTime()
		.Environment
		.ApplicationState
		.Extension
		.Registry
		.CommandRegistry
		.lock()
	{
		Registry.remove(CommandId);
		dev_log!("command-register", "[MountainVinegRPCService] Cocoon unregistered command: {}", CommandId);
	}
}
