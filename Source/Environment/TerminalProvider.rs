//! # TerminalProvider Implementation
//!
//! Implements the `TerminalProvider` trait for the `MountainEnvironment`. This
//! provider contains the core logic for managing integrated terminal instances,
//! including creating native pseudo-terminals (PTYs) and handling their I/O.

use std::{env, io::Write, sync::Arc};

use Common::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::IPCProvider::IPCProvider,
	Terminal::TerminalProvider::TerminalProvider,
};
use async_trait::async_trait;
use log::{error, info, trace, warn};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use serde_json::{Value, json};
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

		let mut TerminalState = TerminalStateDTO::Create(TerminalIdentifier, Name.clone(), &OptionsValue, DefaultShell);

		let PtySystem = NativePtySystem::default();
		let mut PtyPair = PtySystem
			.openpty(PtySize::default())
			.map_err(|e| CommonError::IPCError { Description:format!("Failed to open PTY: {}", e) })?;

		let mut Command = CommandBuilder::new(&TerminalState.ShellPath);
		Command.args(&TerminalState.ShellArguments);
		if let Some(CWD) = &TerminalState.CurrentWorkingDirectory {
			Command.cwd(CWD);
		}

		let mut ChildProcess = PtyPair
			.slave
			.spawn_command(Command)
			.map_err(|e| CommonError::IPCError { Description:format!("Failed to spawn shell process: {}", e) })?;
		TerminalState.OSProcessIdentifier = ChildProcess.process_id();

		let mut PTYWriter = PtyPair.master.take_writer().map_err(|e| {
			CommonError::FileSystemIO {
				Path:"pty master".into(),
				Description:format!("Failed to clone PTY writer: {}", e),
			}
		})?;
		let (InputTransmitter, mut InputReceiver) = TokioMPSC::channel::<String>(32);
		TerminalState.PTYInputTransmitter = Some(InputTransmitter);

		let TermID = TerminalIdentifier;
		tokio::spawn(async move {
			while let Some(Data) = InputReceiver.recv().await {
				if let Err(e) = PTYWriter.write_all(Data.as_bytes()) {
					error!("[TerminalProvider] PTY write failed for ID {}: {}", TermID, e);
					break;
				}
			}
		});

		let mut PTYReader = PtyPair.master.try_clone_reader().map_err(|e| {
			CommonError::FileSystemIO {
				Path:"pty master".into(),
				Description:format!("Failed to clone PTY reader: {}", e),
			}
		})?;
		let IPCProvider:Arc<dyn IPCProvider> = self.Require();
		let TermID = TerminalIdentifier;
		tokio::spawn(async move {
			let mut Buffer = [0u8; 8192];
			loop {
				match PTYReader.read(&mut Buffer) {
					Ok(count) if count > 0 => {
						let DataString = String::from_utf8_lossy(&Buffer[..count]);
						let Payload = json!([TermID, DataString]);
						if let Err(e) = IPCProvider
							.SendNotificationToSidecar(
								"cocoon-main".into(),
								"$acceptTerminalProcessData".into(),
								Payload,
							)
							.await
						{
							warn!("[TerminalProvider] Failed to send process data for ID {}: {}", TermID, e);
						}
					},
					_ => break,
				}
			}
		});

		// Additional tasks for process waiting and cleanup would go here.

		Ok(json!({ "Id": TerminalIdentifier, "Name": Name, "Pid": TerminalState.OSProcessIdentifier }))
	}

	/// Sends text input to a running terminal process.
	async fn SendTextToTerminal(&self, TerminalId:u64, Text:String) -> Result<(), CommonError> {
		trace!("[TerminalProvider] Sending text to terminal ID: {}", TerminalId);

		let SenderOption = {
			let TerminalsGuard = self
				.ApplicationState
				.ActiveTerminals
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;
			if let Some(TerminalArc) = TerminalsGuard.get(&TerminalId) {
				let TerminalStateGuard = TerminalArc.lock().unwrap();
				TerminalStateGuard.PTYInputTransmitter.clone()
			} else {
				None
			}
		}; // Lock is released here

		if let Some(Sender) = SenderOption {
			Sender
				.send(Text)
				.await
				.map_err(|e| CommonError::IPCError { Description:e.to_string() })?;
		}
		Ok(())
	}

	/// Disposes of a terminal instance and terminates its underlying process.
	async fn DisposeTerminal(&self, TerminalId:u64) -> Result<(), CommonError> {
		info!("[TerminalProvider] Disposing terminal ID: {}", TerminalId);
		let TerminalArc = self
			.ApplicationState
			.ActiveTerminals
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.remove(&TerminalId);

		if let Some(TerminalArc) = TerminalArc {
			// Dropping the PTY master and associated tasks will cause the child
			// process to terminate. A more robust implementation might send a
			// SIGHUP or use a platform-specific kill command.
			drop(TerminalArc);
		}
		Ok(())
	}
}
