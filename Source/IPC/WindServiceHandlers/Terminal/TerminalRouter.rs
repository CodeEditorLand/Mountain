//! Terminal command router - delegates all `terminal:*` and `localPty:*` IPC
//! commands.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{Emitter, Manager};
use CommonLibrary::{Environment::Requires::Requires, Storage::StorageProvider::StorageProvider};

use super::*;
use crate::{
	IPC::WindServiceHandlers::{
		Terminal::{
			AttachToProcess::Fn as AttachToProcess,
			DetachFromProcess::Fn as DetachFromProcess,
			LocalPTYCreateProcess::Fn as LocalPTYCreateProcess,
			LocalPTYFreePortKillProcess::Fn as LocalPTYFreePortKillProcess,
			LocalPTYGetDefaultShell::Fn as LocalPTYGetDefaultShell,
			LocalPTYGetEnvironment::Fn as LocalPTYGetEnvironment,
			LocalPTYGetProfiles::Fn as LocalPTYGetProfiles,
			LocalPTYResize::Fn as LocalPTYResize,
			ReviveTerminalProcesses::Fn as ReviveTerminalProcesses,
			SerializeTerminalState::Fn as SerializeTerminalState,
			TerminalCreate::Fn as TerminalCreate,
			TerminalDispose::Fn as TerminalDispose,
			TerminalHide::Fn as TerminalHide,
			TerminalSendText::Fn as TerminalSendText,
			TerminalShow::Fn as TerminalShow,
		},
		Utilities::JsonValueHelpers::{arg_u64, arg_val},
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Routes terminal and localPty commands. Returns Some(result) for handled
/// commands, None otherwise.
pub(crate) async fn route(
	RunTime:Arc<ApplicationRunTime>,

	ApplicationHandle:tauri::AppHandle,

	command:&str,

	Arguments:Vec<Value>,
) -> Option<Result<Value, String>> {
	match command {
		// Terminal commands
		"terminal:create" => {
			dev_log!("terminal", "terminal:create");

			Some(TerminalCreate(RunTime.clone(), Arguments).await)
		},

		"terminal:sendText" => {
			dev_log!("terminal", "terminal:sendText");

			Some(TerminalSendText(RunTime.clone(), Arguments).await)
		},

		"terminal:dispose" => {
			dev_log!("terminal", "terminal:dispose");

			Some(TerminalDispose(RunTime.clone(), Arguments).await)
		},

		"terminal:show" => {
			dev_log!("terminal", "terminal:show");

			Some(TerminalShow(RunTime.clone(), Arguments).await)
		},

		"terminal:hide" => {
			dev_log!("terminal", "terminal:hide");

			Some(TerminalHide(RunTime.clone(), Arguments).await)
		},

		// Local PTY (terminal) commands
		"localPty:getProfiles" => {
			dev_log!("terminal", "localPty:getProfiles");

			Some(LocalPTYGetProfiles().await)
		},

		"localPty:getDefaultSystemShell" => {
			dev_log!("terminal", "localPty:getDefaultSystemShell");

			Some(LocalPTYGetDefaultShell().await)
		},

		// `ILocalPtyService.getTerminalLayoutInfo` - return the last
		// layout snapshot so the workbench restores the terminal panel
		// (active tab, dimensions) across window reloads.
		// Key: "terminal:layoutInfo" in Mountain's `StorageProvider`.
		// `ILocalPtyService.getTerminalLayoutInfo` - return the persisted
		// layout snapshot so the workbench restores the terminal panel
		// (active tab, split dimensions) across window reloads.
		"localPty:getTerminalLayoutInfo" => {
			dev_log!("terminal", "localPty:getTerminalLayoutInfo");

			let StorageProvider:Arc<dyn StorageProvider> = RunTime.Environment.Require();

			let result = match StorageProvider.GetStorageValue(true, "terminal:layoutInfo").await {
				Ok(Some(Stored)) => Ok(Stored),

				Ok(None) => Ok(Value::Null),

				Err(Error) => {
					dev_log!("terminal", "warn: [getTerminalLayoutInfo] storage read failed: {}", Error);

					Ok(Value::Null)
				},
			};

			Some(result)
		},

		// `ILocalPtyService.setTerminalLayoutInfo` - persist the layout
		// snapshot so `getTerminalLayoutInfo` can replay it on next boot.
		"localPty:setTerminalLayoutInfo" => {
			dev_log!("terminal", "localPty:setTerminalLayoutInfo");

			let StorageProvider:Arc<dyn StorageProvider> = RunTime.Environment.Require();

			let Payload = arg_val(&Arguments, 0);

			let _ = StorageProvider
				.UpdateStorageValue(true, "terminal:layoutInfo".to_string(), Some(Payload))
				.await;

			Some(Ok(Value::Null))
		},

		"localPty:getPerformanceMarks" => {
			dev_log!("terminal", "localPty:getPerformanceMarks");

			Some(Ok(json!([])))
		},

		"localPty:reduceConnectionGraceTime" => {
			dev_log!("terminal", "localPty:reduceConnectionGraceTime");

			Some(Ok(Value::Null))
		},

		"localPty:listProcesses" => {
			dev_log!("terminal", "localPty:listProcesses");

			// `IPtyService.listProcesses` returns `IProcessDetails[]`
			// (`vs/platform/terminal/common/terminal.ts`). The
			// workbench uses it for terminal-tab tooltips and the
			// reconnect-on-reload flow. Build entries from the live
			// PTY registry; `isOrphan:false` because Mountain spawns
			// PTYs in-process (they die with us, so there is never a
			// detached pty-host process to revive from).
			let Terminals = RunTime.Environment.ApplicationState.Feature.Terminals.GetAll();

			let mut Entries:Vec<_> = Terminals.values().collect();

			Entries.sort_by_key(|T| T.Identifier);

			let Processes:Vec<Value> = Entries
				.into_iter()
				.map(|T| {
					json!({
						"id": T.Identifier,
						"title": if T.Title.is_empty() { T.Name.clone() } else { T.Title.clone() },
						"titleSource": 0,
						"pid": T.OSProcessIdentifier.unwrap_or(0),
						"cwd": T.GetWorkingDirectory(),
						"workspaceId": "",
						"workspaceName": "",
						"isOrphan": false,
						"icon": Value::Null,
						"color": Value::Null,
						"fixedDimensions": Value::Null,
						"environmentVariableCollections": Value::Null,
						"shellLaunchConfig": {
							"executable": T.ShellPath,
							"args": T.ShellArguments,
						},
						"hasChildProcesses": false,
						"type": Value::Null,
						"hideFromUser": false,
						"isFeatureTerminal": false,
					})
				})
				.collect();

			Some(Ok(json!(Processes)))
		},

		"localPty:getEnvironment" => {
			dev_log!("terminal", "localPty:getEnvironment");

			Some(LocalPTYGetEnvironment().await)
		},

		// `IPtyService.getLatency` (per
		// `vs/platform/terminal/common/terminal.ts:341`) returns
		// `IPtyHostLatencyMeasurement[]`. The workbench polls this
		// to drive its "renderer ↔ pty host" health UI. We have
		// no separate pty host (Mountain spawns PTYs in-process),
		// so latency is effectively zero - return an empty array
		// matching the "no measurements available" branch the
		// workbench already handles. Without this route the call
		// surfaced as `Unknown IPC command: localPty:getLatency`
		// every poll cycle, and the renderer logged a
		// `TauriInvoke ok=false` line per attempt.
		"localPty:getLatency" => {
			dev_log!("terminal", "localPty:getLatency");

			Some(Ok(json!([])))
		},

		// BATCH-19 Part B: VS Code's `LocalPtyService` talks to Mountain via
		// the `localPty:*` channel. The internal implementations reuse the
		// Tauri-side `terminal:*` handlers so PTY lifecycle stays identical
		// regardless of whether the request came from Sky (Wind) or from an
		// extension (Cocoon → Wind channel bridge).
		//
		// CONTRACT NOTE: `IPtyService.createProcess` is typed
		// `Promise<number>` (see `vs/platform/terminal/common/terminal.ts:
		// 316`). The workbench then does `new LocalPty(id, ...)` and
		// `this._ptys.set(id, pty)`. If we return the full
		// `{id,name,pid}` object the renderer keys `_ptys` by that
		// object, every `_ptys.get(<integer>)` lookup from
		// `onProcessData`/`onProcessReady` returns `undefined`, and
		// xterm receives zero bytes - the terminal panel renders
		// blank even though Mountain's PTY reader emits data
		// continuously. Strip down to the integer id here.
		// `localPty:spawn` is Cocoon's Sky bridge path; preserve
		// the full `{id, name, pid}` shape. New `localPty:createProcess`
		// follows VS Code's typed contract.
		"localPty:spawn" => {
			dev_log!("terminal", "localPty:spawn");

			Some(TerminalCreate(RunTime.clone(), Arguments).await)
		},

		"localPty:createProcess" => {
			dev_log!("terminal", "localPty:createProcess");

			Some(LocalPTYCreateProcess(RunTime.clone(), Arguments).await)
		},

		"localPty:start" => {
			// Eager-spawn pattern: `TerminalProvider::CreateTerminal`
			// already started the shell and reader task during
			// `localPty:createProcess`. `start` is a no-op that just
			// completes the workbench's launch promise. Returning
			// `Value::Null` matches `IPtyService.start`'s
			// `Promise<ITerminalLaunchError | ITerminalLaunchResult |
			// undefined>` (`undefined` branch). Routing this back
			// through `TerminalCreate` would spawn a SECOND
			// PTY for the same workbench terminal - the user-visible
			// pane is bound to id=1 from `createProcess`, but a
			// shadow PTY (id=2) starts and streams data nobody
			// renders.
			dev_log!("terminal", "{} no-op (eager-spawn)", command);

			Some(Ok(Value::Null))
		},

		"localPty:input" | "localPty:write" => {
			dev_log!("terminal", "{}", command);

			Some(TerminalSendText(RunTime.clone(), Arguments).await)
		},

		"localPty:shutdown" | "localPty:dispose" => {
			dev_log!("terminal", "{}", command);

			Some(TerminalDispose(RunTime.clone(), Arguments).await)
		},

		"localPty:resize" => {
			dev_log!("terminal", "localPty:resize");

			Some(LocalPTYResize(RunTime.clone(), Arguments).await)
		},

		"localPty:acknowledgeDataEvent" => {
			// xterm flow-control heartbeat; no-op on Mountain side.
			Some(Ok(Value::Null))
		},

		// `ILocalPtyService.getBackendOS` - VS Code uses this to decide
		// which profile list to show (Windows/Linux/macOS). Returns the
		// `OperatingSystem` enum value from
		// `vs/base/common/platform.ts`: 1 = Macintosh, 2 = Linux, 3 = Windows.
		"localPty:getBackendOS" => {
			#[cfg(target_os = "macos")]
			{
				Some(Ok(json!(1)))
			}

			#[cfg(target_os = "linux")]
			{
				Some(Ok(json!(2)))
			}

			#[cfg(target_os = "windows")]
			{
				Some(Ok(json!(3)))
			}

			#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
			{
				Some(Ok(json!(2)))
			}
		},

		// `ILocalPtyService.refreshProperty` - returns the current value
		// of a PTY property. VS Code calls this for `ProcessId` (to show
		// PID in the terminal tab tooltip) and `Cwd` (for smart basename).
		// Property enum: 0=Cwd, 1=ProcessId, 2=Title, 3=OverrideName,
		// 4=ResolvedShellLaunchConfig, 5=ShellType
		"localPty:refreshProperty" => {
			use CommonLibrary::{Environment::Requires::Requires, Terminal::TerminalProvider::TerminalProvider};

			let TerminalId = arg_u64(&Arguments, 0);

			let PropId = arg_u64(&Arguments, 1);

			if TerminalId == 0 {
				Some(Ok(Value::Null))
			} else if PropId == 0 {
				// TerminalProperty::Cwd - return last OSC 633 P;cwd= value
				let Cwd = {
					let guard = RunTime.Environment.ApplicationState.Feature.Terminals.ActiveTerminals.lock();

					guard
						.get(&TerminalId)
						.cloned()
						.and_then(|S| {
							let s_guard = S.lock();

							s_guard.CurrentWorkingDirectory.clone()
						})
						.map(|P| P.to_string_lossy().into_owned())
				};

				Some(Ok(Cwd.map(|C| json!(C)).unwrap_or(Value::Null)))
			} else if PropId == 1 {
				// TerminalProperty::ProcessId
				let Provider:Arc<dyn TerminalProvider> = RunTime.Environment.Require();

				let result = match Provider.GetTerminalProcessId(TerminalId).await {
					Ok(Some(Pid)) => Ok(json!(Pid)),

					_ => Ok(Value::Null),
				};

				Some(result)
			} else {
				Some(Ok(Value::Null))
			}
		},

		// `ILocalPtyService.updateProperty` - workbench notifies Mountain
		// of a property change on a running PTY. Property enum:
		//   2 = Title (dynamic title from shell escape)
		//   3 = OverrideName (user-renamed tab)
		//   5 = ShellType (detected shell identifier)
		// Title / OverrideName are persisted in TerminalStateDTO and
		// forwarded to Sky so the xterm tab label updates live.
		// ShellType is stored for later `refreshProperty` lookups.
		"localPty:updateProperty" => {
			use CommonLibrary::IPC::SkyEvent::SkyEvent;

			let TermId = arg_u64(&Arguments, 0);

			let PropId = arg_u64(&Arguments, 1);

			let PropValue = Arguments.get(2).and_then(Value::as_str).unwrap_or("").to_string();

			if TermId == 0 || PropValue.is_empty() {
				Some(Ok(Value::Null))
			} else {
				match PropId {
					// Title (2) or OverrideName (3): persist + emit to Sky.
					2 | 3 => {
						{
							let Guard = RunTime.Environment.ApplicationState.Feature.Terminals.ActiveTerminals.lock();

							if let Some(Entry) = Guard.get(&TermId) {
								Entry.lock().Title = PropValue.clone();
							}
						}

						dev_log!(
							"terminal",
							"localPty:updateProperty id={} prop={} title='{}'",
							TermId,
							PropId,
							PropValue
						);

						let _ = ApplicationHandle.emit(
							SkyEvent::TerminalPropertyChanged.AsStr(),
							json!({
								"id": TermId,
								"property": PropId,
								"value": PropValue,
							}),
						);
					},

					// ShellType (5): store only; workbench derives its own icon.
					5 => {
						{
							let Guard = RunTime.Environment.ApplicationState.Feature.Terminals.ActiveTerminals.lock();

							if let Some(Entry) = Guard.get(&TermId) {
								Entry.lock().ShellType = Some(PropValue.clone());
							}
						}

						dev_log!("terminal", "localPty:updateProperty id={} shell_type='{}'", TermId, PropValue);
					},

					Other => {
						dev_log!(
							"terminal",
							"localPty:updateProperty id={} unknown_prop={} (no-op)",
							TermId,
							Other
						);
					},
				}

				Some(Ok(Value::Null))
			}
		},

		// `ILocalPtyService.freePortKillProcess` - kill whatever process
		// is listening on a port so a new terminal can bind it.
		"localPty:freePortKillProcess" => {
			dev_log!("terminal", "localPty:freePortKillProcess");

			Some(LocalPTYFreePortKillProcess(Arguments).await)
		},

		// `ILocalPtyService.serializeTerminalProcesses` - snapshot all
		// active terminals so the workbench can persist them to storage
		// and restore them across a window reload. Returns
		// `ISerializedTerminalState[]`.
		"localPty:serializeTerminalState" => {
			dev_log!("terminal", "localPty:serializeTerminalState");

			Some(SerializeTerminalState(RunTime.clone()).await)
		},

		// `ILocalPtyService.reviveTerminalProcesses` - respawn shells from
		// a snapshot produced by `serializeTerminalState`. Accepts
		// `(ISerializedTerminalState[], dateTimeFormatLocale)`.
		"localPty:reviveTerminalProcesses" => {
			dev_log!(
				"terminal",
				"localPty:reviveTerminalProcesses count={}",
				Arguments.first().and_then(|V| V.as_array()).map(|A| A.len()).unwrap_or(0)
			);

			Some(ReviveTerminalProcesses(RunTime.clone(), Arguments).await)
		},

		// `ILocalPtyService.getRevivedPtyNewId` - return the new terminal
		// ID assigned to an old (pre-reload) ID during
		// `reviveTerminalProcesses`. Arguments: `[workspaceId, oldId]`.
		// The mapping is populated by `ReviveTerminalProcesses` and
		// consumed here on first lookup. Falls back to a fresh ID so a
		// missing entry never hangs the workbench.
		"localPty:getRevivedPtyNewId" => {
			let OldId = arg_u64(&Arguments, 1);

			let MaybeNewId = if OldId != 0 {
				RunTime
					.Environment
					.ApplicationState
					.Feature
					.Terminals
					.RevivedIdMap
					.lock()
					.remove(&OldId)
			} else {
				None
			};

			let NewId = MaybeNewId.unwrap_or_else(|| RunTime.Environment.ApplicationState.GetNextTerminalIdentifier());

			dev_log!("terminal", "localPty:getRevivedPtyNewId old_id={} new_id={}", OldId, NewId);

			Some(Ok(json!(NewId)))
		},

		// Session reconnect: reattach the workbench to a live Mountain
		// PTY after a window reload. The provider looks up the terminal
		// by id and returns its PID. DetachFromProcess is the inverse -
		// Mountain keeps the PTY running; output buffer accumulates for
		// the next attach or sky:replay-events drain.
		"localPty:attachToProcess" => {
			dev_log!("terminal", "localPty:attachToProcess");

			Some(AttachToProcess(RunTime.clone(), Arguments).await)
		},

		"localPty:detachFromProcess" => {
			dev_log!("terminal", "localPty:detachFromProcess");

			Some(DetachFromProcess(RunTime.clone(), Arguments).await)
		},

		// `localPty:setActive` - fired by Sky Bridge when the user
		// switches terminal tabs. Notifies Cocoon so that
		// `vscode.window.activeTerminal` reflects the focused terminal.
		"localPty:setActive" => {
			let TermId = Arguments.first().and_then(Value::as_i64);

			let Payload = match TermId {
				Some(Id) => serde_json::json!({ "id": Id }),

				None => serde_json::json!({ "id": null }),
			};

			let _ = crate::Vine::Client::SendNotification::Fn(
				"cocoon-main".to_string(),
				"$acceptActiveTerminalChanged".to_string(),
				Payload,
			)
			.await;

			Some(Ok(Value::Null))
		},

		// `localPty:setShellIntegrationActive` - Sky fires once per
		// terminal when OSC 633 ; A (prompt start) is first detected.
		// Notifies Cocoon so `terminal.shellIntegration !== undefined`
		// and `onDidChangeTerminalShellIntegration` fires.
		"localPty:setShellIntegrationActive" => {
			let TermId = Arguments.first().and_then(Value::as_i64).unwrap_or(0) as u64;

			let _ = crate::Vine::Client::SendNotification::Fn(
				"cocoon-main".to_string(),
				"$acceptTerminalShellIntegrationActivated".to_string(),
				serde_json::json!({ "id": TermId }),
			)
			.await;

			Some(Ok(Value::Null))
		},

		// `localPty:setInteracted` - Sky fires once per terminal when
		// it detects OSC 633 ; B (command-input-begins). Forwards to
		// Cocoon as `$acceptTerminalStateChanged` so subscribers of
		// `vscode.window.onDidChangeTerminalState` see
		// `state.isInteractedWith` flip true. Payload from Sky:
		// `[{ id, interactedWith }]`.
		"localPty:setInteracted" => {
			let Payload = arg_val(&Arguments, 0);

			let _ = crate::Vine::Client::SendNotification::Fn(
				"cocoon-main".to_string(),
				"$acceptTerminalStateChanged".to_string(),
				Payload,
			)
			.await;

			Some(Ok(Value::Null))
		},

		// `localPty:setCwd` - Sky Bridge fires this when it parses an
		// OSC 633 P;cwd=<path> sequence from terminal output. Mountain
		// forwards to Cocoon so `vscode.window.activeTerminal.
		// shellIntegration.cwd` reflects the shell's current directory.
		"localPty:setCwd" => {
			let TermId = Arguments.first().and_then(Value::as_i64).unwrap_or(0) as u64;

			let Cwd = Arguments.get(1).and_then(Value::as_str).unwrap_or("").to_string();

			if !Cwd.is_empty() {
				// Persist CWD in ApplicationState. Lock, update, drop immediately.
				let _CwdPersisted = RunTime
					.Environment
					.ApplicationState
					.Feature
					.Terminals
					.ActiveTerminals
					.lock()
					.get(&TermId)
					.map(|E| {
						E.lock().CurrentWorkingDirectory = Some(std::path::PathBuf::from(&Cwd));
					})
					.is_some();

				let _ = crate::Vine::Client::SendNotification::Fn(
					"cocoon-main".to_string(),
					"$acceptTerminalCwdChange".to_string(),
					serde_json::json!({ "id": TermId, "cwd": Cwd }),
				)
				.await;
			}

			Some(Ok(Value::Null))
		},

		// `localPty:processBinary` - the workbench forwards raw binary
		// input (paste of UTF-16, Ctrl+sequences from xterm.js) here
		// instead of through `input`. Previously dropped to null, which
		// meant pasting from system clipboard or sending Cmd+Shift
		// keyboard escape sequences was silently swallowed. Route
		// through the same TerminalSendText path the input channel
		// uses so the bytes reach the PTY.
		"localPty:processBinary" => {
			dev_log!("terminal", "localPty:processBinary");

			Some(TerminalSendText(RunTime.clone(), Arguments).await)
		},

		// Remaining `localPty:*` - no Mountain-side state needed.
		"localPty:orphanQuestionReply" | "localPty:updateTitle" | "localPty:updateIcon" => Some(Ok(Value::Null)),

		// `ILocalPtyService.installAutoReply` - store an auto-reply rule
		// so the PTY reader can respond automatically to matching output
		// (e.g. password prompts, Y/N confirmations).
		// Payload: `{ answer, match, useCustomAnswer }`.
		"localPty:installAutoReply" => {
			use crate::ApplicationState::State::FeatureState::Terminals::TerminalState::AutoReplyRule;

			let Payload = arg_val(&Arguments, 0);

			let Answer = Payload.get("answer").and_then(Value::as_str).unwrap_or("").to_string();

			let MatchStr = Payload.get("match").and_then(Value::as_str).unwrap_or("").to_string();

			let UseCustom = Payload.get("useCustomAnswer").and_then(Value::as_bool).unwrap_or(false);

			if !Answer.is_empty() && !MatchStr.is_empty() {
				RunTime
					.Environment
					.ApplicationState
					.Feature
					.Terminals
					.AutoReplies
					.lock()
					.push(AutoReplyRule { Match:MatchStr.clone(), Answer:Answer.clone(), UseCustomAnswer:UseCustom });

				dev_log!("terminal", "localPty:installAutoReply match='{}' answer='{}'", MatchStr, Answer);
			}

			Some(Ok(Value::Null))
		},

		// `ILocalPtyService.uninstallAllAutoReplies` - clear every
		// installed auto-reply rule for the current session.
		"localPty:uninstallAllAutoReplies" => {
			RunTime
				.Environment
				.ApplicationState
				.Feature
				.Terminals
				.AutoReplies
				.lock()
				.clear();

			dev_log!("terminal", "localPty:uninstallAllAutoReplies cleared");

			Some(Ok(Value::Null))
		},

		// `localPty:shellExecutionStart` - Sky fires this when it
		// detects OSC 633 ; C (command-output-begins) in terminal
		// data. Payload: `{ id, commandLine, cwd }`. Forward to
		// Cocoon so `vscode.window.onDidStartTerminalShellExecution`
		// subscribers see the execution event. The subscriber lives
		// at `Window/Namespace.ts` on the
		// `window.didStartTerminalShellExecution` Emitter channel.
		"localPty:shellExecutionStart" => {
			let Payload = arg_val(&Arguments, 0);

			let _ = crate::Vine::Client::SendNotification::Fn(
				"cocoon-main".to_string(),
				"$acceptTerminalShellExecutionStart".to_string(),
				Payload,
			)
			.await;

			Some(Ok(Value::Null))
		},

		// `localPty:shellExecutionEnd` - Sky fires this when it
		// detects OSC 633 ; D (command-finished) in terminal data.
		// Payload: `{ id, commandLine, cwd, exitCode }`. Fans to
		// Cocoon as both `$acceptTerminalShellExecutionEnd` (for
		// `onDidEndTerminalShellExecution`) AND a derived
		// `$acceptExecutedTerminalCommand` so
		// `vscode.window.onDidExecuteTerminalCommand` subscribers
		// see the executed command without a separate Sky-side
		// detection pass (the shape is a subset of the end
		// event - same data, different consumer audience).
		"localPty:shellExecutionEnd" => {
			let Payload = arg_val(&Arguments, 0);

			let _ = crate::Vine::Client::SendNotification::Fn(
				"cocoon-main".to_string(),
				"$acceptTerminalShellExecutionEnd".to_string(),
				Payload.clone(),
			)
			.await;

			let _ = crate::Vine::Client::SendNotification::Fn(
				"cocoon-main".to_string(),
				"$acceptExecutedTerminalCommand".to_string(),
				Payload,
			)
			.await;

			Some(Ok(Value::Null))
		},

		_ => None,
	}
}
