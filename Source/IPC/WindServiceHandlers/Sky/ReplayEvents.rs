//! Wire method: `sky:replay-events`.
//! Called by SkyBridge after every `sky://*` Tauri listener is installed.
//! Mountain → Sky `app.emit()` events are NOT buffered: any emit fired before
//! the listener was registered is silently dropped. In the bundled-electron
//! profile, extension activation starts ~580 log lines before the Sky bundle
//! finishes booting (~1995 lines). Without replay, all tree-view + SCM
//! register events are lost and the Activity Bar comes up empty.
//!
//! Replays: tree-views, SCM providers, extension commands, active terminals
//! (including buffered stdout from before SkyBridge's listeners were up),
//! output channels (create + buffered content, capped at the last
//! `OUTPUT_REPLAY_MAX_LINES` lines per channel), diagnostics per owner,
//! and status bar entries.

use std::sync::Arc;

use serde_json::Value;
use tauri::{AppHandle, Emitter};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

/// Per-channel cap on replayed output content. Build logs accumulate
/// megabytes; the workbench only needs the recent tail to repaint the
/// Output panel.
const OUTPUT_REPLAY_MAX_LINES:usize = 200;

pub async fn Fn(ApplicationHandle:AppHandle, RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let mut TreeViewCount:usize = 0;

	let mut ScmCount:usize = 0;

	let mut ScmGroupCount:usize = 0;

	let mut ScmResourceUpdateCount:usize = 0;

	let mut CommandCount:usize = 0;

	let mut TerminalCount:usize = 0;

	let mut TerminalDataBytes:usize = 0;

	// ── Tree views ────────────────────────────────────────────────────────
	let TreeViews = RunTime.Environment.ApplicationState.Feature.TreeViews.ActiveTreeViews.lock();

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

	drop(TreeViews);

	// ── SCM providers ─────────────────────────────────────────────────────
	// Pre-DTO-Identifier-field DTOs default `Identifier` to "" (serde
	// default); fall back to "git" - the only SCM provider in production
	// today is `vscode.git` and a stale state file with empty id is the
	// realistic upgrade-path mismatch.
	let ScmProvidersGuard = RunTime
		.Environment
		.ApplicationState
		.Feature
		.Markers
		.SourceControlManagementProviders
		.lock();

	for (Handle, Dto) in ScmProvidersGuard.iter() {
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

	// ── SCM resource groups ───────────────────────────────────────────────
	// Cocoon's `createResourceGroup(GroupId, Label)` mints
	// `GroupHandle = "${ProviderHandle}/${GroupId}"` and fires
	// `register_scm_resource_group` to Mountain. Replay must reconstruct the
	// same handle so InstallScm's `ScmShimByHandle`/`ScmShimRegistry` lookup
	// resolves the same shim that the live wire path would. Without this
	// replay leg the workbench shows the provider header but zero groups,
	// because `sky://scm/registerGroup` was emitted before SkyBridge's
	// listener was up and Tauri events are not buffered.
	//
	// We resolve `scmId` from the providers map (defaulting to "git" if
	// `Identifier` is empty - matches the provider-replay fallback above).
	let ProviderIdentifierByHandle:std::collections::HashMap<u32, String> = {
		let ScmProviders = RunTime
			.Environment
			.ApplicationState
			.Feature
			.Markers
			.SourceControlManagementProviders
			.lock();

		ScmProviders
			.iter()
			.map(|(Handle, Dto)| {
				let Id = if Dto.Identifier.is_empty() {
					"git".to_string()
				} else {
					Dto.Identifier.clone()
				};

				(*Handle, Id)
			})
			.collect()
	};

	let ScmGroups = RunTime
		.Environment
		.ApplicationState
		.Feature
		.Markers
		.SourceControlManagementGroups
		.lock();

	for (ProviderHandle, GroupsByID) in ScmGroups.iter() {
		let ScmId = ProviderIdentifierByHandle
			.get(ProviderHandle)
			.cloned()
			.unwrap_or_else(|| "git".to_string());

		for (GroupId, GroupDto) in GroupsByID.iter() {
			let GroupHandle = format!("{}/{}", ProviderHandle, GroupId);

			let Payload = serde_json::json!({
				"scmId": ScmId,
				"scmHandle": *ProviderHandle,
				"groupHandle": GroupHandle,
				"groupId": GroupId,
				"label": GroupDto.Label,
			});

			if ApplicationHandle.emit("sky://scm/registerGroup", Payload).is_ok() {
				ScmGroupCount += 1;
			}
		}
	}

	drop(ScmGroups);

	// ── SCM resource updates ──────────────────────────────────────────────
	// After group registration, replay the most recent resource snapshot for
	// each (provider, group) so the workbench's tree populates with the
	// extension's current working-tree state without waiting for the next
	// `update_scm_group` to land. Without this the panel stays empty until
	// the user makes a file change that triggers a fresh group update.
	let ScmResources = RunTime
		.Environment
		.ApplicationState
		.Feature
		.Markers
		.SourceControlManagementResources
		.lock();

	for (ProviderHandle, GroupsByID) in ScmResources.iter() {
		let ScmId = ProviderIdentifierByHandle
			.get(ProviderHandle)
			.cloned()
			.unwrap_or_else(|| "git".to_string());

		for (GroupId, ResourceList) in GroupsByID.iter() {
			let GroupHandle = format!("{}/{}", ProviderHandle, GroupId);

			let Payload = serde_json::json!({
				"scmHandle": *ProviderHandle,
				"providerId": ScmId,
				"groupHandle": GroupHandle,
				"groupId": GroupId,
				"resourceStates": ResourceList,
			});

			if ApplicationHandle.emit("sky://scm/updateGroup", Payload).is_ok() {
				ScmResourceUpdateCount += 1;
			}
		}
	}

	drop(ScmResources);

	// ── Extension commands ────────────────────────────────────────────────
	// Emit ONE batched event with the whole array. Per-command emits
	// (one per registered command, ~1000+ during extension boot) saturate
	// Tauri's shared WKWebView IPC channel and starve keystroke delivery.
	// SkyBridge accepts `{ commands: [...] }` or `{ id, commandId, kind }`.
	let Commands = RunTime.Environment.ApplicationState.Extension.Registry.CommandRegistry.lock();

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

	drop(Commands);

	if !Batch.is_empty() {
		let Count = Batch.len();

		if ApplicationHandle
			.emit("sky://command/register", serde_json::json!({ "commands": Batch }))
			.is_ok()
		{
			CommandCount = Count;
		}
	}

	// ── Terminals + buffered stdout ───────────────────────────────────────
	// Each active terminal needs its `create` event AND any buffered stdout
	// the PTY reader produced before SkyBridge was up. Without this, the
	// shell's first prompt is silently dropped and the user sees an empty
	// terminal pane until they type.
	let Terminals = RunTime.Environment.ApplicationState.Feature.Terminals.ActiveTerminals.lock();

	for (TerminalId, Arc) in Terminals.iter() {
		let (Name, Pid) = {
			let State = Arc.lock();

			(State.Name.clone(), State.OSProcessIdentifier.unwrap_or(0))
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

	for (TerminalId, Bytes) in crate::Environment::TerminalProvider::Fn() {
		let DataString = String::from_utf8_lossy(&Bytes).to_string();

		TerminalDataBytes += Bytes.len();

		let _ = ApplicationHandle.emit(
			"sky://terminal/data",
			serde_json::json!({ "id": TerminalId, "data": DataString }),
		);
	}

	// ── Output channels (create + buffered content) ──────────────────────
	// `Feature.OutputChannels` retains each channel's name, buffered text
	// and visibility (populated by the OutputProvider and the generic
	// notification rail's output handlers). Replay re-creates the channel,
	// then pushes the buffered tail through `sky://output/replace` -
	// atomic in Sky's bridge, so content delivered live after the
	// listeners came up is not duplicated.
	let mut OutputChannelCount:usize = 0;

	let mut OutputReplayBytes:usize = 0;

	let OutputChannels = RunTime.Environment.ApplicationState.Feature.OutputChannels.GetAll();

	for (ChannelId, Dto) in OutputChannels.iter() {
		let Name = if Dto.Name.is_empty() { ChannelId.clone() } else { Dto.Name.clone() };

		if ApplicationHandle
			.emit("sky://output/create", serde_json::json!({ "id": ChannelId, "name": Name }))
			.is_ok()
		{
			OutputChannelCount += 1;
		}

		if Dto.Buffer.is_empty() {
			continue;
		}

		let Tail = {
			let Lines:Vec<&str> = Dto.Buffer.split_inclusive('\n').collect();

			if Lines.len() > OUTPUT_REPLAY_MAX_LINES {
				Lines[Lines.len() - OUTPUT_REPLAY_MAX_LINES..].concat()
			} else {
				Dto.Buffer.clone()
			}
		};

		OutputReplayBytes += Tail.len();

		let _ = ApplicationHandle.emit(
			"sky://output/replace",
			serde_json::json!({ "channel": ChannelId, "value": Tail }),
		);

		if Dto.IsVisible {
			let _ = ApplicationHandle.emit("sky://output/show", serde_json::json!({ "channel": ChannelId }));
		}
	}

	// ── Diagnostics (markers per owner) ───────────────────────────────────
	// `Feature.Diagnostics` retains the full owner → URI → markers map.
	// Re-emit one `sky://diagnostics/changed` per owner in the same
	// payload shape `DiagnosticProvider::SetDiagnostics` produces, so
	// SkyBridge's marker bridge repopulates `IMarkerService` (squiggles +
	// Problems panel) without waiting for the next language-server push.
	let mut DiagnosticsOwnerCount:usize = 0;

	let mut DiagnosticsUriCount:usize = 0;

	let Diagnostics = RunTime.Environment.ApplicationState.Feature.Diagnostics.GetAll();

	for (Owner, MarkersByUri) in Diagnostics.iter() {
		if MarkersByUri.is_empty() {
			continue;
		}

		let Uris:Vec<String> = MarkersByUri.keys().cloned().collect();

		let ChangedEntries:Vec<serde_json::Value> = MarkersByUri
			.iter()
			.map(|(Uri, Markers)| serde_json::json!({ "uri": Uri, "markers": Markers }))
			.collect();

		let Payload = serde_json::json!({
			"Owner": Owner,
			"owner": Owner,
			"Uris": Uris,
			"changedURIs": ChangedEntries,
		});

		if ApplicationHandle.emit("sky://diagnostics/changed", Payload).is_ok() {
			DiagnosticsOwnerCount += 1;

			DiagnosticsUriCount += MarkersByUri.len();
		}
	}

	// ── Status bar entries ────────────────────────────────────────────────
	// `Feature.Markers.ActiveStatusBarItems` retains every entry pushed
	// through `SetStatusBarEntry`. Replay mirrors the live payload shape
	// of `set_status_bar_entry_impl` so SkyBridge's `SetOrUpdateEntry`
	// upserts into `IStatusbarService` identically.
	let mut StatusBarCount:usize = 0;

	let StatusBarItems = RunTime.Environment.ApplicationState.Feature.Markers.GetStatusBarItems();

	for (EntryId, Entry) in StatusBarItems.iter() {
		let Payload = serde_json::json!({
			"id": EntryId,
			"itemId": Entry.ItemIdentifier,
			"extensionId": Entry.ExtensionIdentifier,
			"name": Entry.Name,
			"text": Entry.Text,
			"tooltip": Entry.Tooltip,
			"command": Entry.Command,
			"color": Entry.Color,
			"backgroundColor": Entry.BackgroundColor,
			"alignment": if Entry.IsAlignedLeft { 0 } else { 1 },
			"priority": Entry.Priority,
			"accessibilityInformation": Entry.AccessibilityInformation,
		});

		if ApplicationHandle.emit("sky://statusbar/set-entry", Payload).is_ok() {
			StatusBarCount += 1;
		}
	}

	crate::dev_log!(
		"sky-emit",
		"[SkyEmit] replay-events tree-views={} scm={} scm-groups={} scm-resource-updates={} commands={} terminals={} \
		 terminal-bytes={} output-channels={} output-bytes={} diagnostics-owners={} diagnostics-uris={} statusbar={}",
		TreeViewCount,
		ScmCount,
		ScmGroupCount,
		ScmResourceUpdateCount,
		CommandCount,
		TerminalCount,
		TerminalDataBytes,
		OutputChannelCount,
		OutputReplayBytes,
		DiagnosticsOwnerCount,
		DiagnosticsUriCount,
		StatusBarCount
	);

	Ok(serde_json::json!({
		"treeViews": TreeViewCount,
		"scmProviders": ScmCount,
		"scmGroups": ScmGroupCount,
		"scmResourceUpdates": ScmResourceUpdateCount,
		"commands": CommandCount,
		"terminals": TerminalCount,
		"terminalDataBytes": TerminalDataBytes,
		"outputChannels": OutputChannelCount,
		"outputReplayBytes": OutputReplayBytes,
		"diagnosticsOwners": DiagnosticsOwnerCount,
		"diagnosticsUris": DiagnosticsUriCount,
		"statusBarItems": StatusBarCount,
	}))
}
