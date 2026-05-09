#![allow(non_snake_case)]
//! Cocoon → Mountain `registerCommand` notification.
//! Stores the command as a `Proxied` handler in Mountain's
//! `CommandRegistry` so subsequent `commands.executeCommand` calls get
//! routed back to Cocoon via `$executeContributedCommand` gRPC. The
//! sidecar identifier is hard-coded to `cocoon-main` because that is
//! the sole extension-host Cocoon instance today.

use std::{
	sync::{
		Arc,
		Mutex,
		OnceLock,
		atomic::{AtomicBool, Ordering},
	},
	time::Duration,
};

use serde_json::{Value, json};

use tauri::{AppHandle, Emitter};

use crate::{
	Environment::CommandProvider::CommandHandler,
	Vine::Server::MountainVinegRPCService::MountainVinegRPCService,
	dev_log,
};

/// Coalesced Mountain → Sky emit buffer for `sky://command/register`.
///
/// Extension boot fires 1000+ `registerCommand` notifications in a
/// tight burst (113 extensions × ~10 commands each). Emitting one
/// Tauri event per command saturated the WKWebView IPC channel that
/// also carries keystroke delivery; users could type for a split
/// second before the burst hit, then nothing. Buffer for one frame
/// (16 ms) and emit a single `{ commands: [...] }` batch instead.
/// SkyBridge's listener accepts both shapes (single + batch).
struct CommandEmitBatch {

	Pending:Mutex<Vec<Value>>,

	FlushScheduled:AtomicBool,
}

static COMMAND_EMIT_BATCH:OnceLock<Arc<CommandEmitBatch>> = OnceLock::new();

fn EnqueueCommandEmit(Handle:&AppHandle, Payload:Value) {

	let Batch = COMMAND_EMIT_BATCH.get_or_init(|| {
		Arc::new(CommandEmitBatch { Pending:Mutex::new(Vec::new()), FlushScheduled:AtomicBool::new(false) })
	});

	{

		let mut Pending = Batch.Pending.lock().unwrap();

		Pending.push(Payload);
	}

	if !Batch.FlushScheduled.swap(true, Ordering::AcqRel) {

		let BatchClone = Batch.clone();

		let HandleClone = Handle.clone();

		tokio::spawn(async move {
			tokio::time::sleep(Duration::from_millis(16)).await;
			let Drained:Vec<Value> = {
				let mut Pending = BatchClone.Pending.lock().unwrap();
				std::mem::take(&mut *Pending)
			};
			BatchClone.FlushScheduled.store(false, Ordering::Release);
			if Drained.is_empty() {
				return;
			}
			let Count = Drained.len();
			match HandleClone.emit("sky://command/register", json!({ "commands": Drained })) {
				Ok(()) => {
					dev_log!("sky-emit", "[SkyEmit] ok channel=sky://command/register batch={}", Count);
				},
				Err(Error) => {
					dev_log!(
						"sky-emit",

						"[SkyEmit] fail channel=sky://command/register batch={} error={}",

						Count,

						Error
					);
				},
			}
		});
	}
}

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

	let Kind = Parameter.get("kind").and_then(Value::as_str).unwrap_or("command").to_string();

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

	// Coalesce the Sky emit. SkyBridge listens on `sky://command/register`
	// and accepts either `{ id, commandId, kind }` (single) or
	// `{ commands: [...] }` (batch). The batched flush happens 16 ms
	// after the first command lands, so an extension-boot burst of 1000+
	// registrations becomes a single Tauri emit instead of 1000.
	EnqueueCommandEmit(
		Service.ApplicationHandle(),

		json!({ "id": CommandId, "commandId": CommandId, "kind": Kind }),
	);
}
