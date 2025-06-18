// @module TerminalLogic
// @description Contains the core logic for managing integrated terminal
// instances, including creating native pseudo-terminals (PTYs) and handling
// their I/O.

use std::{
	env,
	io::Write,
	sync::{Arc, Mutex as StdMutex},
};

use Common::error::CommonError;
use log::{error, info, trace, warn};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime};
use tokio::{
	io::AsyncReadExt,
	sync::{Mutex as TokioMutex, mpsc as TokioMpsc},
};

use crate::{
	ApplicationState::{ApplicationState::ApplicationState, DTO::TerminalStateDTO},
	Vine::client,
};

// Logic to create a new terminal instance. This is called by the
// `TerminalProvider` in the Environment.
pub async fn CreateTerminalLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	options_value:Value,
) -> Result<Value, CommonError> {
	let app_state = app_handle.state::<ApplicationState>();
	let terminal_id = app_state.GetNextTerminalIdentifier();

	let default_shell = if cfg!(windows) {
		"powershell.exe".to_string()
	} else {
		env!("SHELL").to_string()
	};
	let name = options_value
		.get("name")
		.and_then(Value::as_str)
		.unwrap_or("terminal")
		.to_string();

	info!("[TerminalLogic] Creating terminal ID: {}, Name: '{}'", terminal_id, name);

	let mut terminal_state = TerminalStateDTO::New(terminal_id, name.clone(), &options_value, default_shell);

	let pty_system = NativePtySystem::default();
	let pty_pair = pty_system
		.openpty(PtySize::default())
		.map_err(|e| CommonError::IpcError { Description:format!("Failed to open PTY: {}", e) })?;

	let mut command = CommandBuilder::new(&terminal_state.ShellPath);
	command.args(&terminal_state.ShellArgument);
	if let Some(cwd) = &terminal_state.CurrentWorkingDirectory {
		command.cwd(cwd);
	}
	if let Some(env) = &terminal_state.EnvironmentVariables {
		for (key, value) in env {
			if let Some(val) = value {
				command.env(key, val);
			} else {
				command.env_remove(key);
			}
		}
	}

	let mut child_process = pty_pair
		.slave
		.spawn_command(command)
		.map_err(|e| CommonError::IpcError { Description:format!("Failed to spawn shell: {}", e) })?;
	terminal_state.OsProcessIdentifier = child_process.process_id();

	let mut pty_writer = pty_pair.master.try_clone_writer().map_err(|e| {
		CommonError::IoError { Path:"pty master".into(), Description:format!("Failed to clone writer: {}", e) }
	})?;
	let (input_tx, mut input_rx) = TokioMpsc::channel::<String>(32);
	terminal_state.PtyInputTransmitter = Some(input_tx);

	// --- Spawn I/O and lifecycle management tasks ---
	let writer_id = terminal_id;
	tokio::spawn(async move {
		// Writer Task: Listens on the MPSC channel and writes data to the PTY.
		while let Some(data) = input_rx.recv().await {
			if let Err(e) = pty_writer.write_all(data.as_bytes()) {
				error!("[TerminalWriter] PTY write failed for ID {}: {}", writer_id, e);
				break;
			}
		}
		trace!("[TerminalWriter] Writer task for ID {} finished.", writer_id);
	});

	let mut pty_reader = pty_pair.master.try_clone_reader().map_err(|e| {
		CommonError::IoError { Path:"pty master".into(), Description:format!("Failed to clone reader: {}", e) }
	})?;
	let reader_id = terminal_id;
	let reader_task_handle = tokio::spawn(async move {
		// Reader Task: Reads data from the PTY and sends it to Cocoon.
		let mut buffer = [0u8; 8192];
		loop {
			match pty_reader.read(&mut buffer).await {
				Ok(Some(count)) if count > 0 => {
					let data_str = String::from_utf8_lossy(&buffer[..count]);
					let payload = json!([reader_id, data_str]);
					if let Err(e) =
						client::SendNotification("cocoon-main".into(), "$acceptTerminalProcessData".into(), payload)
							.await
					{
						warn!("[TerminalReader] Failed to send process data for ID {}: {}", reader_id, e);
					}
				},
				Ok(Some(_)) => {},          // 0 bytes read, continue
				Ok(None) | Err(_) => break, // EOF or error
			}
		}
		trace!("[TerminalReader] Reader task for ID {} finished.", reader_id);
	});

	let waiter_app_handle = app_handle.clone();
	let waiter_id = terminal_id;
	let waiter_task_handle = tokio::spawn(async move {
		// Process Waiter Task: Awaits the shell process termination.
		let exit_status = child_process.wait().unwrap_or_default();
		info!(
			"[TerminalWaiter] Terminal ID {} exited with status: {:?}",
			waiter_id, exit_status
		);
		client::SendNotification(
			"cocoon-main",
			"$acceptTerminalClosed".into(),
			json!([waiter_id, exit_status.code()]),
		)
		.await
		.unwrap_or_else(|e| {
			warn!(
				"[TerminalWaiter] Failed to send closed notification for ID {}: {}",
				waiter_id, e
			)
		});
		waiter_app_handle
			.state::<ApplicationState>()
			.ActiveTerminals
			.lock()
			.unwrap()
			.remove(&waiter_id);
		trace!("[TerminalWaiter] Waiter task for ID {} finished.", waiter_id);
	});

	terminal_state.ReaderTaskHandle = Some(Arc::new(TokioMutex::new(Some(reader_task_handle))));
	terminal_state.ProcessWaitHandle = Some(Arc::new(TokioMutex::new(Some(waiter_task_handle))));

	app_state
		.ActiveTerminals
		.lock()
		.unwrap()
		.insert(terminal_id, Arc::new(StdMutex::new(terminal_state.clone())));

	// Notify Cocoon about the new terminal
	client::SendNotification(
		"cocoon-main",
		"$acceptTerminalOpened".into(),
		json!([terminal_id, name.clone()]),
	)
	.await
	.unwrap_or_else(|e| warn!("[TerminalLogic] Failed to send opened notification: {}", e));
	if let Some(pid) = terminal_state.OsProcessIdentifier {
		client::SendNotification("cocoon-main", "$acceptTerminalProcessId".into(), json!([terminal_id, pid]))
			.await
			.unwrap_or_else(|e| warn!("[TerminalLogic] Failed to send PID notification: {}", e));
	}

	Ok(json!({ "Id": terminal_id, "Name": name, "Pid": terminal_state.OsProcessIdentifier }))
}

// Logic to send text input to a terminal process.
pub async fn SendTextToTerminalLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	terminal_id:u64,
	text:String,
) -> Result<(), CommonError> {
	trace!("[TerminalLogic] Sending text to terminal ID: {}", terminal_id);
	let app_state = app_handle.state::<ApplicationState>();
	let terminals_guard = app_state.ActiveTerminals.lock().unwrap();
	if let Some(terminal_arc) = terminals_guard.get(&terminal_id) {
		let terminal_state_guard = terminal_arc.lock().unwrap();
		if let Some(sender) = &terminal_state_guard.PtyInputTransmitter {
			sender
				.send(text)
				.await
				.map_err(|e| CommonError::IpcError { Description:e.to_string() })?;
		} else {
			return Err(CommonError::IpcError {
				Description:format!("Terminal with ID {} has no input channel.", terminal_id),
			});
		}
	} else {
		return Err(CommonError::IpcError { Description:format!("Terminal with ID {} not found.", terminal_id) });
	}
	Ok(())
}

// Logic to dispose of a terminal instance.
pub async fn DisposeTerminalLogic<R:Runtime>(app_handle:&AppHandle<R>, terminal_id:u64) -> Result<(), CommonError> {
	info!("[TerminalLogic] Disposing terminal ID: {}", terminal_id);
	let app_state = app_handle.state::<ApplicationState>();
	if let Some(terminal_arc) = app_state.ActiveTerminals.lock().unwrap().remove(&terminal_id) {
		let mut terminal_state_guard = terminal_arc.lock().unwrap();
		// Abort the background tasks associated with this terminal.
		// Taking the handles ensures they are only aborted once.
		if let Some(handle_arc) = terminal_state_guard.ReaderTaskHandle.take() {
			if let Some(handle) = handle_arc.lock().await.take() {
				handle.abort();
			}
		}
		if let Some(handle_arc) = terminal_state_guard.ProcessWaitHandle.take() {
			if let Some(handle) = handle_arc.lock().await.take() {
				handle.abort();
			}
		}
		// The writer task will terminate automatically when the receiver
		// (PtyInputTransmitter) is dropped.
	} else {
		warn!(
			"[TerminalLogic] Attempted to dispose of non-existent terminal ID: {}",
			terminal_id
		);
	}
	Ok(())
}
