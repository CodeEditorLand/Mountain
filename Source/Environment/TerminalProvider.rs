//! File: Mountain/Source/Environment/TerminalProvider.rs
//! Role: Implements the `TerminalProvider` trait for the `MountainEnvironment`.
//! Responsibilities:
//!   - Core logic for managing integrated terminal instances.
//!   - Creating native pseudo-terminals (PTYs) and handling their I/O.
//!   - Spawning and managing the lifecycle of the underlying shell processes.
//!   - Handle terminal show/hide UI state.
//!   - Send text input to terminal processes.
//!   - Manage terminal environment variables.
//!   - Handle terminal resizing and dimension management.
//!   -Support terminal profiles and configuration.
//!   - Handle terminal process exit detection.
//!   - Manage terminal input/output channels.
//!   - Support terminal color schemes and themes.
//!   - Handle terminal bell/notification support.
//!   - Implement terminal buffer management.
//!   - Support terminal search and navigation.
//!   - Handle terminal clipboard operations.
//!   - Implement terminal tab support.
//!   - Support custom shell integration.
//!
//! TODOs:
//!   - Implement terminal profile management
//!   - Add terminal environment variable management
//!   - Implement terminal resize handling (PtySize updates)
//!   - Support terminal color scheme configuration
//!   - Add terminal bell handling and visual notifications
//!   - Implement terminal buffer scrolling and history
//!   - Support terminal search within output
//!   - Add terminal reconnection for crashed processes
//!   - Implement terminal tab management
//!   - Support terminal split view
//!   - Add terminal decoration support (e.g., cwd indicator)
//!   - Implement terminal command history
//!   - Support terminal shell integration (e.g., fish, zsh, bash)
//!   - Add terminal ANSI escape sequence handling
//!   - Implement terminal clipboard operations
//!   - Support terminal link detection and navigation
//!   - Add terminal performance optimizations for large output
//!   - Implement terminal process tree (parent/child processes)
//!   - Support terminal environment injection
//!   - Add terminal keyboard mapping customization
//!   - Implement terminal logging for debugging
//!   - Support terminal font size and font family
//!   - Add terminal UTF-8 and Unicode support
//!   - Implement terminal timeout and idle detection
//!   - Support terminal command execution automation
//!   - Add terminal multi-instance management
//!
//! Inspired by VSCode's integrated terminal which:
//! - Uses native PTY for process isolation
//! - Streams I/O to avoid blocking the main thread
//! - Supports multiple terminal instances
//! - Handles terminal show/hide state
//! - Manages terminal process lifecycle
//! - Supports terminal profiles and custom shells
//! - Provides shell integration features
//! # TerminalProvider Implementation
//!
//! Implements the `TerminalProvider` trait for the `MountainEnvironment`. This
//! provider contains the core logic for managing integrated terminal instances,
//! including creating native pseudo-terminals (PTYs) and handling their I/O.
//!
//! ## Terminal Architecture
//!
//! The terminal implementation uses the following architecture:
//!
//! 1. **PTY Creation**: Use `portable-pty` to create native PTY pairs
//! 2. **Process Spawning**: Spawn shell process as child of PTY slave
//! 3. **I/O Streaming**: Spawn async tasks for input and output streaming
//! 4. **IPC Communication**: Forward output to Cocoon sidecar via IPC
//! 5. **State Management**: Track terminal state in ApplicationState
//!
//! ## Terminal Lifecycle
//!
//! 1. **Create**: Create PTY, spawn shell, start I/O tasks
//! 2. **SendText**: Write user input to PTY master
//! 3. **ReceiveData**: Read output from PTY and forward to sidecar
//! 4. **Show/Hide**: Emit UI events to show/hide terminal
//! 5. **ProcessExit**: Detect shell exit and notify sidecar
//! 6. **Dispose**: Close PTY, kill process, cleanup state
//!
//! ## Shell Detection
//!
//! Default shell selection by platform:
//! - **Windows**: `powershell.exe`
//! - **macOS/Linux**: `$SHELL` environment variable, fallback to `sh`
//!
//! Custom shell paths can be provided via terminal options.
//!
//! ## I/O Streaming
//!
//! Terminal I/O is handled by background tokio tasks:
//!
//! - **Input Task**: Receives text from channel and writes to PTY master
//! - **Output Task**: Reads from PTY master and forwards to sidecar
//! - **Exit Task**: Waits for process exit and notifies sidecar
//!
//! Each terminal gets its own I/O tasks to prevent blocking each other.

use std::{env, io::Write, sync::Arc};

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::{IPCProvider::IPCProvider, SkyEvent::SkyEvent},
	Terminal::TerminalProvider::TerminalProvider,
};
use async_trait::async_trait;
use portable_pty::{CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem};
use serde_json::{Value, json};
use tauri::Emitter;
use tokio::sync::mpsc as TokioMPSC;

use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::{ApplicationState::DTO::TerminalStateDTO::TerminalStateDTO, IPC::SkyEmit::LogSkyEmit, dev_log};

// Per-terminal recent-output buffer. The PTY reader task races SkyBridge's
// `listen("sky://terminal/data", ...)` install: in the bundled-electron
// profile, the shell's first prompt + any startup chatter (zsh's MOTD,
// `direnv` exports, fish's greeting, …) fires within ~50 ms of
// `localPty:createProcess` while Sky's bundle is still parsing for ~1500 ms.
// Without buffering, those bytes vanish and the user sees an empty pane
// until they type something to coax fresh output. We buffer up to
// `MAX_BUFFERED_BYTES` per terminal and replay on `sky:replay-events`.
//
// The buffer is bounded; on overflow we drop oldest bytes (keep the most
// recent suffix). 64 KB is enough for ~600 lines of typical zsh/bash
// startup; tail-cropping preserves the prompt the user actually needs to
// see.
const MAX_BUFFERED_BYTES:usize = 64 * 1024;

static TERMINAL_OUTPUT_BUFFER:std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<u64, Vec<u8>>>> =
	std::sync::OnceLock::new();

fn TerminalOutputBuffer() -> &'static std::sync::Mutex<std::collections::HashMap<u64, Vec<u8>>> {
	TERMINAL_OUTPUT_BUFFER.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

pub fn AppendTerminalOutput(TerminalId:u64, Bytes:&[u8]) {
	if let Ok(mut Map) = TerminalOutputBuffer().lock() {
		let Entry = Map.entry(TerminalId).or_insert_with(Vec::new);
		Entry.extend_from_slice(Bytes);
		// Drop oldest if over cap. Keep the trailing MAX_BUFFERED_BYTES so
		// the prompt + most-recent context survive.
		if Entry.len() > MAX_BUFFERED_BYTES {
			let DropCount = Entry.len() - MAX_BUFFERED_BYTES;
			Entry.drain(..DropCount);
		}
	}
}

pub fn DrainTerminalOutputBuffer() -> Vec<(u64, Vec<u8>)> {
	if let Ok(Map) = TerminalOutputBuffer().lock() {
		Map.iter().map(|(K, V)| (*K, V.clone())).collect()
	} else {
		Vec::new()
	}
}

pub fn RemoveTerminalOutputBuffer(TerminalId:u64) {
	if let Ok(mut Map) = TerminalOutputBuffer().lock() {
		Map.remove(&TerminalId);
	}
}

#[async_trait]
impl TerminalProvider for MountainEnvironment {
	/// Creates a new terminal instance, spawns a PTY, and manages its I/O.
	async fn CreateTerminal(&self, OptionsValue:Value) -> Result<Value, CommonError> {
		let TerminalIdentifier = self.ApplicationState.GetNextTerminalIdentifier();

		let DefaultShell = if cfg!(windows) {
			"powershell.exe".to_string()
		} else {
			env::var("SHELL").unwrap_or_else(|_| "sh".to_string())
		};

		let Name = OptionsValue
			.get("name")
			.and_then(Value::as_str)
			.unwrap_or("terminal")
			.to_string();

		dev_log!(
			"terminal",
			"[TerminalProvider] Creating terminal ID: {}, Name: '{}'",
			TerminalIdentifier,
			Name
		);

		let mut TerminalState = TerminalStateDTO::Create(TerminalIdentifier, Name.clone(), &OptionsValue, DefaultShell)
			.map_err(|e| {
				CommonError::ConfigurationLoad { Description:format!("Failed to create terminal state: {}", e) }
			})?;

		let PtySystem = NativePtySystem::default();

		let PtyPair = PtySystem
			.openpty(PtySize::default())
			.map_err(|Error| CommonError::IPCError { Description:format!("Failed to open PTY: {}", Error) })?;

		let mut Command = CommandBuilder::new(&TerminalState.ShellPath);

		Command.args(&TerminalState.ShellArguments);

		if let Some(CWD) = &TerminalState.CurrentWorkingDirectory {
			Command.cwd(CWD);
		}

		let mut ChildProcess = PtyPair.slave.spawn_command(Command).map_err(|Error| {
			CommonError::IPCError { Description:format!("Failed to spawn shell process: {}", Error) }
		})?;

		TerminalState.OSProcessIdentifier = ChildProcess.process_id();

		let mut PTYWriter = PtyPair.master.take_writer().map_err(|Error| {
			CommonError::FileSystemIO {
				Path:"pty master".into(),

				Description:format!("Failed to take PTY writer: {}", Error),
			}
		})?;

		let (InputTransmitter, mut InputReceiver) = TokioMPSC::channel::<String>(32);

		TerminalState.PTYInputTransmitter = Some(InputTransmitter);

		let TermIDForInput = TerminalIdentifier;

		tokio::spawn(async move {
			while let Some(Data) = InputReceiver.recv().await {
				if let Err(Error) = PTYWriter.write_all(Data.as_bytes()) {
					dev_log!(
						"terminal",
						"error: [TerminalProvider] PTY write failed for ID {}: {}",
						TermIDForInput,
						Error
					);

					break;
				}
			}
		});

		let mut PTYReader = PtyPair.master.try_clone_reader().map_err(|Error| {
			CommonError::FileSystemIO {
				Path:"pty master".into(),

				Description:format!("Failed to clone PTY reader: {}", Error),
			}
		})?;

		// Keep the master PTY alive past `CreateTerminal` so `ResizeTerminal`
		// can call `resize()` on it and so dropping it during `DisposeTerminal`
		// tears the shell down cleanly.
		let PTYMasterHandle:crate::ApplicationState::DTO::TerminalStateDTO::PtyMasterHandle =
			Arc::new(std::sync::Mutex::new(PtyPair.master));
		TerminalState.PTYMaster = Some(PTYMasterHandle);

		let IPCProvider:Arc<dyn IPCProvider> = self.Require();

		let TermIDForOutput = TerminalIdentifier;
		let AppHandleForOutput = self.ApplicationHandle.clone();

		tokio::spawn(async move {
			let mut Buffer = [0u8; 8192];

			loop {
				match PTYReader.read(&mut Buffer) {
					Ok(count) if count > 0 => {
						// Buffer the bytes for replay-on-late-listener. The
						// SkyBridge install completes ~1500 ms after Cocoon
						// activates, and the shell's first prompt fires
						// immediately after `spawn_command`. Without a
						// buffer the prompt is silently lost and the user
						// sees an empty terminal pane until they type.
						AppendTerminalOutput(TermIDForOutput, &Buffer[..count]);

						let DataString = String::from_utf8_lossy(&Buffer[..count]).to_string();

						// Fan out in two directions so both consumers see
						// the bytes:
						//   1. Cocoon's extension host (via gRPC) - lets
						//      `vscode.window.onDidWriteTerminalData` and the SCM
						//      `$acceptTerminalProcessData` chain continue to function.
						//   2. Sky's webview (via Tauri event) - the UI xterm renderer subscribes to
						//      `sky://terminal/data` and draws the bytes into the user-visible terminal
						//      panel.
						// Without the Tauri emit the user sees a terminal
						// panel open but no shell output because gRPC-only
						// delivery bypasses the webview entirely (BATCH-19
						// Part B).
						let Payload = json!([TermIDForOutput, DataString.clone()]);
						if let Err(Error) = IPCProvider
							.SendNotificationToSideCar(
								"cocoon-main".into(),
								"$acceptTerminalProcessData".into(),
								Payload,
							)
							.await
						{
							dev_log!(
								"terminal",
								"warn: [TerminalProvider] Failed to send process data for ID {}: {}",
								TermIDForOutput,
								Error
							);
						}

						if let Err(Error) = AppHandleForOutput.emit(
							SkyEvent::TerminalData.AsStr(),
							json!({
								"id": TermIDForOutput,
								"data": DataString,
							}),
						) {
							dev_log!(
								"terminal",
								"warn: [TerminalProvider] sky://terminal/data emit failed for ID {}: {}",
								TermIDForOutput,
								Error
							);
						}
					},

					// Break on Ok(0) or Err
					_ => break,
				}
			}
		});

		let TermIDForExit = TerminalIdentifier;

		// BATCH-19 Part B: capture the PID before `ChildProcess` is moved into
		// the exit-watcher task so the exit log line can correlate with the
		// spawn log (`[TerminalProvider] localPty:spawn OK id=N pid=M`). Also
		// surface the actual exit status code - previously discarded via
		// `let _exit_status = …`, which meant the log could only say "has
		// exited" without distinguishing a clean `exit 0`, `echo hi; exit`
		// flow from a crash. That distinction is what the BATCH-19 smoke test
		// needs to confirm the shell really ran and returned.
		let PidForExit = ChildProcess.process_id();

		let EnvironmentClone = self.clone();

		tokio::spawn(async move {
			let ExitStatus = ChildProcess.wait();

			// portable-pty's `Child::wait()` returns `io::Result<ExitStatus>`.
			// `{:?}` on ExitStatus shows `success` and any captured code
			// without needing to commit to a specific accessor name (the
			// crate's exit-status API has varied across versions).
			let StatusSummary = match &ExitStatus {
				Ok(Code) => format!("exited {:?}", Code),
				Err(Error) => format!("wait failed: {}", Error),
			};

			dev_log!(
				"terminal",
				"[TerminalProvider] Process for terminal ID {} pid={:?} {}",
				TermIDForExit,
				PidForExit,
				StatusSummary
			);

			let IPCProvider:Arc<dyn IPCProvider> = EnvironmentClone.Require();

			if let Err(Error) = IPCProvider
				.SendNotificationToSideCar(
					"cocoon-main".into(),
					"$acceptTerminalProcessExit".into(),
					json!([TermIDForExit]),
				)
				.await
			{
				dev_log!(
					"terminal",
					"warn: [TerminalProvider] Failed to send process exit notification for ID {}: {}",
					TermIDForExit,
					Error
				);
			}

			// Clean up the terminal from the state
			if let Ok(mut Guard) = EnvironmentClone.ApplicationState.Feature.Terminals.ActiveTerminals.lock() {
				Guard.remove(&TermIDForExit);
			}
			// Drop the recent-output replay buffer; nothing left to replay
			// after the shell has exited.
			RemoveTerminalOutputBuffer(TermIDForExit);

			// Tell Sky the xterm panel should drop - mirrors the `sky://`
			// create emit above. Without this, the UI keeps a ghost panel
			// after the shell exits (user types `exit` and the pane still
			// lingers until the next render cycle).
			if let Err(Error) = LogSkyEmit(
				&EnvironmentClone.ApplicationHandle,
				SkyEvent::TerminalExit.AsStr(),
				json!({ "id": TermIDForExit }),
			) {
				dev_log!(
					"terminal",
					"warn: [TerminalProvider] sky://terminal/exit emit failed for ID {}: {}",
					TermIDForExit,
					Error
				);
			}
		});

		self.ApplicationState
			.Feature
			.Terminals
			.ActiveTerminals
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.insert(TerminalIdentifier, Arc::new(std::sync::Mutex::new(TerminalState.clone())));

		// BATCH-19 Part B: let Sky render the new terminal panel without
		// waiting for Cocoon to round-trip a notification. The `sky://` event
		// channel is already how ShowTerminal / HideTerminal talk to the UI.
		//
		// RACE FIX: emit on a deferred tokio task (~120 ms) instead of
		// synchronously. The workbench's `LocalTerminalBackend.createProcess`
		// flow is:
		//   1. await this._proxy.createProcess(...)   // RPC IN-FLIGHT
		//   2. const pty = new LocalPty(id, …)        // POST-await
		//   3. this._ptys.set(id, pty)                // POST-await
		// The patched `_connectToDirectProxy` listener for
		// `_localPtyService.onProcessReady` does
		// `this._ptys.get(e.id)?.handleReady(e.event)`. If we emit
		// synchronously while CreateTerminal is still inside step (1),
		// the Tauri event fires before step (3) - `_ptys.get(id)` returns
		// `undefined`, `handleReady` is skipped, `BasePty._onProcessReady`
		// never fires, `processManager._onProcessReady` never fires,
		// `ptyProcessReady` never resolves - and every `processManager.
		// write(data)` call (which `terminalInstance._handleOnData`
		// `await`s) hangs forever. The user sees the panel render but
		// every keystroke is silently dropped because `LocalPty.input`
		// is never reached. A 120 ms delay gives the RPC response
		// roundtrip + `_ptys.set` plenty of headroom on real hardware.
		// Same race applies to `sky://terminal/data` for the shell's
		// first prompt - the existing `AppendTerminalOutput` replay
		// buffer covers data, but the create event needs explicit
		// deferral because there's no replay path for ready.
		let CreateAppHandle = self.ApplicationHandle.clone();
		let CreateTermId = TerminalIdentifier;
		let CreateName = Name.clone();
		let CreatePid = TerminalState.OSProcessIdentifier;
		tokio::spawn(async move {
			tokio::time::sleep(std::time::Duration::from_millis(120)).await;
			let CreatePayload = json!({
				"id": CreateTermId,
				"name": CreateName,
				"pid": CreatePid,
			});
			// `LogSkyEmit` makes the deferred emit visible under
			// `[DEV:SKY-EMIT]` so the next log dissection can confirm
			// the deferral landed (and how many `localPty:input` calls
			// arrived afterwards). The bare `.emit()` we replaced was
			// invisible to the histogram.
			if let Err(Error) = LogSkyEmit(&CreateAppHandle, SkyEvent::TerminalCreate.AsStr(), CreatePayload) {
				dev_log!(
					"terminal",
					"warn: [TerminalProvider] sky://terminal/create emit failed for ID {}: {}",
					CreateTermId,
					Error
				);
			}
		});

		dev_log!(
			"terminal",
			"[TerminalProvider] localPty:spawn OK id={} pid={:?}",
			TerminalIdentifier,
			TerminalState.OSProcessIdentifier
		);

		Ok(json!({ "id": TerminalIdentifier, "name": Name, "pid": TerminalState.OSProcessIdentifier }))
	}

	async fn SendTextToTerminal(&self, TerminalId:u64, Text:String) -> Result<(), CommonError> {
		dev_log!("terminal", "[TerminalProvider] Sending text to terminal ID: {}", TerminalId);

		let SenderOption = {
			let TerminalsGuard = self
				.ApplicationState
				.Feature
				.Terminals
				.ActiveTerminals
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

			TerminalsGuard
				.get(&TerminalId)
				.and_then(|TerminalArc| TerminalArc.lock().ok())
				.and_then(|TerminalStateGuard| TerminalStateGuard.PTYInputTransmitter.clone())
		};

		if let Some(Sender) = SenderOption {
			Sender
				.send(Text)
				.await
				.map_err(|Error| CommonError::IPCError { Description:Error.to_string() })
		} else {
			Err(CommonError::IPCError {
				Description:format!("Terminal with ID {} not found or has no input channel.", TerminalId),
			})
		}
	}

	async fn DisposeTerminal(&self, TerminalId:u64) -> Result<(), CommonError> {
		dev_log!("terminal", "[TerminalProvider] Disposing terminal ID: {}", TerminalId);

		let TerminalArc = self
			.ApplicationState
			.Feature
			.Terminals
			.ActiveTerminals
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.remove(&TerminalId);

		if let Some(TerminalArc) = TerminalArc {
			// Dropping the PTY master's writer and reader handles will signal the
			// underlying process to terminate.
			drop(TerminalArc);
		}

		Ok(())
	}

	async fn ShowTerminal(&self, TerminalId:u64, PreserveFocus:bool) -> Result<(), CommonError> {
		dev_log!("terminal", "[TerminalProvider] Showing terminal ID: {}", TerminalId);

		self.ApplicationHandle
			.emit(
				SkyEvent::TerminalShow.AsStr(),
				json!({ "id": TerminalId, "preserveFocus": PreserveFocus }),
			)
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })
	}

	async fn HideTerminal(&self, TerminalId:u64) -> Result<(), CommonError> {
		dev_log!("terminal", "[TerminalProvider] Hiding terminal ID: {}", TerminalId);

		// Low-frequency lifecycle event - safe to route through
		// `LogSkyEmit` for histogram visibility.
		LogSkyEmit(
			&self.ApplicationHandle,
			SkyEvent::TerminalHide.AsStr(),
			json!({ "id": TerminalId }),
		)
		.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })
	}

	async fn GetTerminalProcessId(&self, TerminalId:u64) -> Result<Option<u32>, CommonError> {
		let TerminalsGuard = self
			.ApplicationState
			.Feature
			.Terminals
			.ActiveTerminals
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		Ok(TerminalsGuard
			.get(&TerminalId)
			.and_then(|t| t.lock().ok().and_then(|g| g.OSProcessIdentifier)))
	}

	async fn ResizeTerminal(&self, TerminalId:u64, Columns:u16, Rows:u16) -> Result<(), CommonError> {
		if Columns == 0 || Rows == 0 {
			return Err(CommonError::InvalidArgument {
				ArgumentName:"Columns/Rows".to_string(),
				Reason:format!("Columns and Rows must be ≥ 1 (got {}×{})", Columns, Rows),
			});
		}

		// Pull the shared master-PTY handle out of the state lock before touching
		// it so we never hold the outer terminals map while performing IO.
		let MasterOption = {
			let TerminalsGuard = self
				.ApplicationState
				.Feature
				.Terminals
				.ActiveTerminals
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;
			TerminalsGuard
				.get(&TerminalId)
				.and_then(|TerminalArc| TerminalArc.lock().ok())
				.and_then(|TerminalStateGuard| TerminalStateGuard.PTYMaster.clone())
		};

		let Master = MasterOption.ok_or_else(|| {
			CommonError::IPCError {
				Description:format!("Terminal with ID {} not found or has no PTY master handle.", TerminalId),
			}
		})?;

		let Size = PtySize { rows:Rows, cols:Columns, pixel_width:0, pixel_height:0 };

		// Method resolution walks through MutexGuard → Box → dyn MasterPty,
		// so `Guard.resize(...)` dispatches straight to the trait impl. Keep
		// the call inside `spawn_blocking` even though portable-pty's resize
		// is nominally fast - SIGWINCH delivery can stall briefly when the
		// child shell is ptrace-frozen or mid-syscall.
		tokio::task::spawn_blocking(move || {
			let Guard = Master.lock().map_err(|_| "PTY master mutex poisoned".to_string())?;
			Guard.resize(Size).map_err(|Error| Error.to_string())
		})
		.await
		.map_err(|Error| CommonError::IPCError { Description:format!("resize join error: {}", Error) })?
		.map_err(|Error| CommonError::IPCError { Description:format!("PTY resize failed: {}", Error) })?;

		dev_log!(
			"terminal",
			"[TerminalProvider] Resized terminal ID {} to {}×{}",
			TerminalId,
			Columns,
			Rows
		);

		Ok(())
	}
}
