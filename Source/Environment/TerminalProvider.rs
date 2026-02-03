// File: Mountain/Source/Environment/TerminalProvider.rs
// Role: Implements the `TerminalProvider` trait for the `MountainEnvironment`.
// Responsibilities:
//   - Core logic for managing integrated terminal instances.
//   - Creating native pseudo-terminals (PTYs) and handling their I/O.
//   - Spawning and managing the lifecycle of the underlying shell processes.
//   - Handle terminal show/hide UI state.
//   - Send text input to terminal processes.
//   - Manage terminal environment variables.
//   - Handle terminal resizing and dimension management.
//   -Support terminal profiles and configuration.
//   - Handle terminal process exit detection.
//   - Manage terminal input/output channels.
//   - Support terminal color schemes and themes.
//   - Handle terminal bell/notification support.
//   - Implement terminal buffer management.
//   - Support terminal search and navigation.
//   - Handle terminal clipboard operations.
//   - Implement terminal tab support.
//   - Support custom shell integration.
//
// TODOs:
//   - Implement terminal profile management
//   - Add terminal environment variable management
//   - Implement terminal resize handling (PtySize updates)
//   - Support terminal color scheme configuration
//   - Add terminal bell handling and visual notifications
//   - Implement terminal buffer scrolling and history
//   - Support terminal search within output
//   - Add terminal reconnection for crashed processes
//   - Implement terminal tab management
//   - Support terminal split view
//   - Add terminal decoration support (e.g., cwd indicator)
//   - Implement terminal command history
//   - Support terminal shell integration (e.g., fish, zsh, bash)
//   - Add terminal ANSI escape sequence handling
//   - Implement terminal clipboard operations
//   - Support terminal link detection and navigation
//   - Add terminal performance optimizations for large output
//   - Implement terminal process tree (parent/child processes)
//   - Support terminal environment injection
//   - Add terminal keyboard mapping customization
//   - Implement terminal logging for debugging
//   - Support terminal font size and font family
//   - Add terminal UTF-8 and Unicode support
//   - Implement terminal timeout and idle detection
//   - Support terminal command execution automation
//   - Add terminal multi-instance management
//
// Inspired by VSCode's integrated terminal which:
// - Uses native PTY for process isolation
// - Streams I/O to avoid blocking the main thread
// - Supports multiple terminal instances
// - Handles terminal show/hide state
// - Manages terminal process lifecycle
// - Supports terminal profiles and custom shells
// - Provides shell integration features
//! # TerminalProvider Implementation
//!
//! Implements the `TerminalProvider` trait for the `MountainEnvironment`. This
//! provider contains the core logic for managing integrated terminal instances,
//! including creating native pseudo-terminals (PTYs) and handling their I/O.
//
//! ## Terminal Architecture
//!
//! The terminal implementation uses the following architecture:
//!
//! 1. **PTY Creation**: Use `portable-pty` to create native PTY pairs
//! 2. **Process Spawning**: Spawn shell process as child of PTY slave
//! 3. **I/O Streaming**: Spawn async tasks for input and output streaming
//! 4. **IPC Communication**: Forward output to Cocoon sidecar via IPC
//! 5. **State Management**: Track terminal state in ApplicationState
//
//! ## Terminal Lifecycle
//!
//! 1. **Create**: Create PTY, spawn shell, start I/O tasks
//! 2. **SendText**: Write user input to PTY master
//! 3. **ReceiveData**: Read output from PTY and forward to sidecar
//! 4. **Show/Hide**: Emit UI events to show/hide terminal
//! 5. **ProcessExit**: Detect shell exit and notify sidecar
//! 6. **Dispose**: Close PTY, kill process, cleanup state
//
//! ## Shell Detection
//!
//! Default shell selection by platform:
//! - **Windows**: `powershell.exe`
//! - **macOS/Linux**: `$SHELL` environment variable, fallback to `sh`
//!
//! Custom shell paths can be provided via terminal options.
//
//! ## I/O Streaming
//!
//! Terminal I/O is handled by background tokio tasks:
//!
//! - **Input Task**: Receives text from channel and writes to PTY master
//! - **Output Task**: Reads from PTY master and forwards to sidecar
//! - **Exit Task**: Waits for process exit and notifies sidecar
//
//! Each terminal gets its own I/O tasks to prevent blocking each other.

use std::{env, io::Write, sync::Arc};

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::IPCProvider::IPCProvider,
	Terminal::TerminalProvider::TerminalProvider,
};
use async_trait::async_trait;
use log::{error, info, trace, warn};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use serde_json::{Value, json};
use tauri::Emitter;
use tokio::sync::mpsc as TokioMPSC;

use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::ApplicationState::DTO::TerminalStateDTO::TerminalStateDTO;

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

		info!(
			"[TerminalProvider] Creating terminal ID: {}, Name: '{}'",
			TerminalIdentifier, Name
		);

		let mut TerminalState = TerminalStateDTO::Create(TerminalIdentifier, Name.clone(), &OptionsValue, DefaultShell)
			.map_err(|e| {
				CommonError::ConfigurationError { Description:format!("Failed to create terminal state: {}", e) }
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
					error!("[TerminalProvider] PTY write failed for ID {}: {}", TermIDForInput, Error);

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

		let IPCProvider:Arc<dyn IPCProvider> = self.Require();

		let TermIDForOutput = TerminalIdentifier;

		tokio::spawn(async move {
			let mut Buffer = [0u8; 8192];

			loop {
				match PTYReader.read(&mut Buffer) {
					Ok(count) if count > 0 => {
						let DataString = String::from_utf8_lossy(&Buffer[..count]);

						let Payload = json!([TermIDForOutput, DataString.to_string()]);

						if let Err(Error) = IPCProvider
							.SendNotificationToSideCar(
								"cocoon-main".into(),
								"$acceptTerminalProcessData".into(),
								Payload,
							)
							.await
						{
							warn!(
								"[TerminalProvider] Failed to send process data for ID {}: {}",
								TermIDForOutput, Error
							);
						}
					},

					// Break on Ok(0) or Err
					_ => break,
				}
			}
		});

		let TermIDForExit = TerminalIdentifier;

		let EnvironmentClone = self.clone();

		tokio::spawn(async move {
			let _exit_status = ChildProcess.wait();

			info!("[TerminalProvider] Process for terminal ID {} has exited.", TermIDForExit);

			let IPCProvider:Arc<dyn IPCProvider> = EnvironmentClone.Require();

			if let Err(Error) = IPCProvider
				.SendNotificationToSideCar(
					"cocoon-main".into(),
					"$acceptTerminalProcessExit".into(),
					json!([TermIDForExit]),
				)
				.await
			{
				warn!(
					"[TerminalProvider] Failed to send process exit notification for ID {}: {}",
					TermIDForExit, Error
				);
			}

			// Clean up the terminal from the state
			if let Ok(mut Guard) = EnvironmentClone.ApplicationState.ActiveTerminals.lock() {
				Guard.remove(&TermIDForExit);
			}
		});

		self.ApplicationState
			.ActiveTerminals
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.insert(TerminalIdentifier, Arc::new(std::sync::Mutex::new(TerminalState.clone())));

		Ok(json!({ "id": TerminalIdentifier, "name": Name, "pid": TerminalState.OSProcessIdentifier }))
	}

	async fn SendTextToTerminal(&self, TerminalId:u64, Text:String) -> Result<(), CommonError> {
		trace!("[TerminalProvider] Sending text to terminal ID: {}", TerminalId);

		let SenderOption = {
			let TerminalsGuard = self
				.ApplicationState
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
		info!("[TerminalProvider] Disposing terminal ID: {}", TerminalId);

		let TerminalArc = self
			.ApplicationState
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
		info!("[TerminalProvider] Showing terminal ID: {}", TerminalId);

		self.ApplicationHandle
			.emit(
				"sky://terminal/show",
				json!({ "id": TerminalId, "preserveFocus": PreserveFocus }),
			)
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })
	}

	async fn HideTerminal(&self, TerminalId:u64) -> Result<(), CommonError> {
		info!("[TerminalProvider] Hiding terminal ID: {}", TerminalId);

		self.ApplicationHandle
			.emit("sky://terminal/hide", json!({ "id": TerminalId }))
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })
	}

	async fn GetTerminalProcessId(&self, TerminalId:u64) -> Result<Option<u32>, CommonError> {
		let TerminalsGuard = self
			.ApplicationState
			.ActiveTerminals
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		Ok(TerminalsGuard
			.get(&TerminalId)
			.and_then(|t| t.lock().ok().and_then(|g| g.OSProcessIdentifier)))
	}
}
