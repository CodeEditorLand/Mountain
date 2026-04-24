#![allow(non_snake_case)]
//! Cocoon → Mountain `registerCommand` notification.
//! Stores the command as a `Proxied` handler in Mountain's
//! `CommandRegistry` so subsequent `commands.executeCommand` calls get
//! routed back to Cocoon via `$executeContributedCommand` gRPC. The
//! sidecar identifier is hard-coded to `cocoon-main` because that is
//! the sole extension-host Cocoon instance today.

use serde_json::Value;

use crate::{
	Environment::CommandProvider::CommandHandler,
	Vine::Server::MountainVinegRPCService::MountainVinegRPCService,
	dev_log,
};

pub async fn RegisterCommand(Service:&MountainVinegRPCService, Parameter:&Value) {
	let CommandId = Parameter.get("commandId").and_then(Value::as_str).unwrap_or("");
	// Per-command registration (~100 commands / session). Useful for
	// verifying extension command contributions but noisy at the `grpc`
	// level. Route to `command-register` so it's opt-in alongside
	// `provider-register`.
	dev_log!("command-register", "[MountainVinegRPCService] Cocoon registered command: {}", CommandId);
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
		Registry.insert(
			CommandId.to_string(),
			CommandHandler::Proxied {
				SideCarIdentifier:"cocoon-main".to_string(),
				CommandIdentifier:CommandId.to_string(),
			},
		);
	}
}
