#![allow(unused_variables, dead_code, unused_imports)]

//! Notification: `registerCommand`.

use serde_json::Value;
use CommonLibrary::Command::CommandExecutor::CommandExecutor;

use crate::{Environment::MountainEnvironment::MountainEnvironment, dev_log};

pub async fn Fn(Params:Value, Env:&MountainEnvironment) {
	let CommandId = Params.get("commandId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	if let Err(Error) = Env.RegisterCommand(ExtensionId, CommandId.clone()).await {
		dev_log!(
			"cocoon",
			"warn: [CocoonService] notification: registerCommand '{}' failed: {:?}",
			CommandId,
			Error
		);
	}
}
