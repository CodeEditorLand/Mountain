#![allow(unused_variables, dead_code, unused_imports)]

//! Notification: `unregisterCommand`.

use serde_json::Value;
use CommonLibrary::Command::CommandExecutor::CommandExecutor;

use crate::Environment::MountainEnvironment::MountainEnvironment;

pub async fn Fn(Params:Value, Env:&MountainEnvironment) {
	let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let CommandId = Params.get("commandId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = Env.UnregisterCommand(ExtensionId, CommandId).await;
}
