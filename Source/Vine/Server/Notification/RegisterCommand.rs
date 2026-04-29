#![allow(non_snake_case)]
//! Cocoon → Mountain `registerCommand` notification.
//! Stores the command as a `Proxied` handler in Mountain's
//! `CommandRegistry` so subsequent `commands.executeCommand` calls get
//! routed back to Cocoon via `$executeContributedCommand` gRPC. The
//! sidecar identifier is hard-coded to `cocoon-main` because that is
//! the sole extension-host Cocoon instance today.

use serde_json::{Value, json};

use crate::{
	Environment::CommandProvider::CommandHandler,
	IPC::SkyEmit::LogSkyEmit,
	Vine::Server::MountainVinegRPCService::MountainVinegRPCService,
	dev_log,
};

pub async fn RegisterCommand(Service:&MountainVinegRPCService, Parameter:&Value) {
	let CommandId = Parameter.get("commandId").and_then(Value::as_str).unwrap_or("");
	// Per-command registration (~100 commands / session). Useful for
	// verifying extension command contributions but noisy at the `grpc`
	// level. Route to `command-register` so it's opt-in alongside
	// `provider-register`.
	dev_log!(
		"command-register",
		"[MountainVinegRPCService] Cocoon registered command: {}",
		CommandId
	);
	if CommandId.is_empty() {
		return;
	}
	let Kind = Parameter
		.get("kind")
		.and_then(Value::as_str)
		.unwrap_or("command")
		.to_string();
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
	// Sky's `SkyBridge.ts:824` listens on `sky://command/register` to
	// surface the command in the workbench `ICommandService` registry -
	// without this emit, every extension-contributed command was added
	// to Mountain's registry but invisible to the command palette /
	// keybinding-resolver. Payload shape matches the Sky destructure
	// (`{ id }` or `{ commandId }` - both probed).
	// Convert to `LogSkyEmit` so command-register volume is observable
	// in the `[DEV:SKY-EMIT]` histogram. Extension command registration
	// is bursty - 100+ commands per session - so the channel count
	// gives a quick read on whether contributions are landing.
	let _ = LogSkyEmit(
		Service.ApplicationHandle(),
		"sky://command/register",
		json!({ "id": CommandId, "commandId": CommandId, "kind": Kind }),
	);
}
