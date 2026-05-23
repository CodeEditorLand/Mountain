
//! Wire method: `sky:replay-events`.
//! Called by SkyBridge after every `sky://*` Tauri listener is installed.
//! Mountain → Sky `app.emit()` events are NOT buffered: any emit fired before
//! the listener was registered is silently dropped. In the bundled-electron
//! profile, extension activation starts ~580 log lines before the Sky bundle
//! finishes booting (~1995 lines). Without replay, all tree-view + SCM
//! register events are lost and the Activity Bar comes up empty.
//!
//! Replays: tree-views, SCM providers, extension commands, active terminals
//! (including buffered stdout from before SkyBridge's listeners were up).

use std::sync::Arc;

use serde_json::Value;
use tauri::{AppHandle, Emitter};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(ApplicationHandle:AppHandle, RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let mut TreeViewCount:usize = 0;

	let mut ScmCount:usize = 0;

	let mut CommandCount:usize = 0;

	let mut TerminalCount:usize = 0;

	let mut TerminalDataBytes:usize = 0;

	// ── Tree views ────────────────────────────────────────────────────────
	if let Ok(TreeViews) = RunTime.Environment.ApplicationState.Feature.TreeViews.ActiveTreeViews.lock() {
		for (ViewId, Dto) in TreeViews.iter() {
			let Payload = serde_json::json!({
				"viewId": ViewId,
				"options": {
					"canSelectMany": Dto.CanSelectMany,
					"showCollapseAll": Dto.HasHandleDrag,
					"title": Dto.Title.clone().unwrap_or_default(),
				},
			});

			if ApplicationHandle.emit("sky://tree-view/create", Payload).is_ok() {
				TreeViewCount += 1;
			}
		}
	}

	// ── SCM providers ─────────────────────────────────────────────────────
	// Pre-DTO-Identifier-field DTOs default `Identifier` to "" (serde
	// default); fall back to "git" - the only SCM provider in production
	// today is `vscode.git` and a stale state file with empty id is the
	// realistic upgrade-path mismatch.
	if let Ok(ScmProviders) = RunTime
		.Environment
		.ApplicationState
		.Feature
		.Markers
		.SourceControlManagementProviders
		.lock()
	{
		for (Handle, Dto) in ScmProviders.iter() {
			let RootUriStr = Dto
				.RootURI
				.as_ref()
				.and_then(|V| V.get("external").or_else(|| V.get("path")))
				.and_then(serde_json::Value::as_str)
				.unwrap_or("")
				.to_string();

			let ScmId = if Dto.Identifier.is_empty() {
				"git".to_string()
			} else {
				Dto.Identifier.clone()
			};

			let Payload = serde_json::json!({
				"scmId": ScmId,
				"label": Dto.Label,
				"rootUri": RootUriStr,
				"extensionId": "",
				"handle": *Handle,
			});

			if ApplicationHandle.emit("sky://scm/register", Payload).is_ok() {
				ScmCount += 1;
			}
		}
	}

	// ── Extension commands ────────────────────────────────────────────────
	// Emit ONE batched event with the whole array. Per-command emits
	// (one per registered command, ~1000+ during extension boot) saturate
	// Tauri's shared WKWebView IPC channel and starve keystroke delivery.
	// SkyBridge accepts `{ commands: [...] }` or `{ id, commandId, kind }`.
	if let Ok(Commands) = RunTime.Environment.ApplicationState.Extension.Registry.CommandRegistry.lock() {
		let mut Batch:Vec<serde_json::Value> = Vec::new();

		for (CommandId, Handler) in Commands.iter() {
			use crate::Environment::CommandProvider::CommandHandler;

			let Kind = match Handler {
				CommandHandler::Native(_) => continue,

				CommandHandler::Proxied { .. } => "extension",
			};

			Batch.push(serde_json::json!({
				"id": CommandId,
				"commandId": CommandId,
				"kind": Kind,
			}));
		}

		if !Batch.is_empty() {
			let Count = Batch.len();

			if ApplicationHandle
				.emit("sky://command/register", serde_json::json!({ "commands": Batch }))
				.is_ok()
			{
				CommandCount = Count;
			}
		}
	}

	// ── Terminals + buffered stdout ───────────────────────────────────────
	// Each active terminal needs its `create` event AND any buffered stdout
	// the PTY reader produced before SkyBridge was up. Without this, the
	// shell's first prompt is silently dropped and the user sees an empty
	// terminal pane until they type.
	if let Ok(Terminals) = RunTime.Environment.ApplicationState.Feature.Terminals.ActiveTerminals.lock() {
		for (TerminalId, Arc) in Terminals.iter() {
			let (Name, Pid) = if let Ok(State) = Arc.lock() {
				(State.Name.clone(), State.OSProcessIdentifier.unwrap_or(0))
			} else {
				(String::new(), 0)
			};

			let CreatePayload = serde_json::json!({
				"id": *TerminalId,
				"name": Name,
				"pid": Pid,
			});

			if ApplicationHandle.emit("sky://terminal/create", CreatePayload).is_ok() {
				TerminalCount += 1;
			}
		}
	}

	for (TerminalId, Bytes) in crate::Environment::TerminalProvider::Fn() {
		let DataString = String::from_utf8_lossy(&Bytes).to_string();

		TerminalDataBytes += Bytes.len();

		let _ = ApplicationHandle.emit(
			"sky://terminal/data",
			serde_json::json!({ "id": TerminalId, "data": DataString }),
		);
	}

	crate::dev_log!(
		"sky-emit",
		"[SkyEmit] replay-events tree-views={} scm={} commands={} terminals={} terminal-bytes={}",
		TreeViewCount,
		ScmCount,
		CommandCount,
		TerminalCount,
		TerminalDataBytes
	);

	Ok(serde_json::json!({
		"treeViews": TreeViewCount,
		"scmProviders": ScmCount,
		"commands": CommandCount,
		"terminals": TerminalCount,
		"terminalDataBytes": TerminalDataBytes,
	}))
}
