use std::{
	io::Write,
	sync::{Arc, Mutex as StdMutex},
};

use Common::error::CommonError;
use log::{debug, error, info};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use serde_json::{Value, json};
use tauri::{ApplicationHandle, Emitter, Manager, RunTime};
use tokio::{
	io::AsyncReadExt,
	sync::{Mutex as TokioMutex, mpsc as TokioMpsc},
};

// @module TerminalLogic
// @description Contains the core logic for managing integrated terminal
// instances, including creating native pseudo-terminals (PTYs) and handling
// their I/O.
use crate::{
	ApplicationState::{ApplicationState::ApplicationState, DTO::TerminalStateDto},
	vine::{self, client},
};

// Logic to create a new terminal instance. This is called by the
// `TerminalProvider` in the environment.
pub async fn CreateTerminalLogic<R:RunTime>(ApplicationHandle:&ApplicationHandle<R>, OptionsValue:Value) -> Result<Value, CommonError> {
	let AppStateInstance = ApplicationHandle.state::<ApplicationState>();
	let TerminalId = AppStateInstance.GetNextTerminalIdentifier();
	let DefaultShell = if cfg!(windows) { "cmd.exe".to_string() } else { env!("SHELL").to_string() };
	let Name = OptionsValue
		.get("Name")
		.and_then(Value::as_str)
		.unwrap_or("terminal")
		.to_string();

	info!("[TerminalLogic] Creating terminal ID: {}, Name: '{}'", TerminalId, Name);

	let mut TerminalState = TerminalStateDto::New(TerminalId, Name.clone(), &OptionsValue, DefaultShell);

	let PtySystem = NativePtySystem::default();
	let PtyPair = PtySystem
		.openpty(PtySize::default())
		.map_err(|e| CommonError::IpcError { Description:format!("Failed to open PTY: {}", e) })?;

	let mut Command = CommandBuilder::new(&TerminalState.ShellPath);
	// ... configure CommandBuilder ...

	let mut ChildProcess = PtyPair
		.slave
		.spawn_command(Command)
		.map_err(|e| CommonError::IpcError { Description:format!("Failed to spawn shell: {}", e) })?;
	TerminalState.OsProcessIdentifier = ChildProcess.process_id();

	let mut PtyWriter = PtyPair
		.master
		.try_clone_writer()
		.map_err(|_| CommonError::IoError { Path:"pty master".into(), Description:"Failed to clone writer".into() })?;
	let (InputTx, mut InputRx) = TokioMpsc::channel::<String>(32);
	TerminalState.PtyInputTransmitter = Some(InputTx);

	// --- Spawn I/O and lifecycle management tasks ---
	tokio::spawn(async move {
		// Writer Task
		while let Some(Data) = InputRx.recv().await {
			if let Err(e) = PtyWriter.write_all(Data.as_bytes()) {
				error!("[TerminalWriter] PTY write failed for ID {}: {}", TerminalId, e);
				break;
			}
		}
	});

	let mut PtyReader = PtyPair
		.master
		.try_clone_reader()
		.map_err(|_| CommonError::IoError { Path:"pty master".into(), Description:"Failed to clone reader".into() })?;
	let ReaderApplicationHandle = ApplicationHandle.clone();
	let ReaderTaskHandle = tokio::spawn(async move {
		// Reader Task
		let mut Buffer = [0u8; 8192];
		loop {
			match PtyReader.read(&mut Buffer).await {
				Ok(Some(Count)) if Count > 0 => {
					let DataStr = String::from_utf8_lossy(&Buffer[..Count]);
					let Payload = json!([TerminalId, DataStr]);
					client::SendNotification("cocoon-main", "$acceptTerminalProcessData".into(), Payload)
						.await
						.ok();
				},
				_ => break, // EOF or error
			}
		}
	});

	let WaiterApplicationHandle = ApplicationHandle.clone();
	let WaiterTaskHandle = tokio::spawn(async move {
		// Process Waiter Task
		let ExitStatus = ChildProcess.wait().unwrap_or_default();
		info!(
			"[TerminalWaiter] Terminal ID {} exited with status: {:?}",
			TerminalId, ExitStatus
		);
		client::SendNotification(
			"cocoon-main",
			"$acceptTerminalClosed".into(),
			json!([TerminalId, ExitStatus.code()]),
		)
		.await
		.ok();
		WaiterApplicationHandle
			.state::<ApplicationState>()
			.ActiveTerminals
			.lock()
			.unwrap()
			.remove(&TerminalId);
	});

	TerminalState.ReaderTaskHandle = Some(Arc::new(TokioMutex::new(Some(ReaderTaskHandle))));
	TerminalState.ProcessWaitHandle = Some(Arc::new(TokioMutex::new(Some(WaiterTaskHandle))));

	AppStateInstance
		.ActiveTerminals
		.lock()
		.unwrap()
		.insert(TerminalId, Arc::new(StdMutex::new(TerminalState.clone())));

	// Notify Cocoon about the new terminal
	client::SendNotification("cocoon-main", "$acceptTerminalOpened".into(), json!([TerminalId, Name.clone()]))
		.await
		.ok();
	if let Some(Pid) = TerminalState.OsProcessIdentifier {
		client::SendNotification("cocoon-main", "$acceptTerminalProcessId".into(), json!([TerminalId, Pid]))
			.await
			.ok();
	}

	Ok(json!({ "Id": TerminalId, "Name": Name, "Pid": TerminalState.OsProcessIdentifier }))
}

// Logic to send text input to a terminal process.
pub async fn SendTextToTerminalLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	TerminalId:u64,
	Text:String,
) -> Result<(), CommonError> {
	info!("[TerminalLogic] Sending text to terminal ID: {}", TerminalId);
	let AppStateInstance = ApplicationHandle.state::<ApplicationState>();
	let TerminalsGuard = AppStateInstance.ActiveTerminals.lock().unwrap();
	if let Some(TerminalArc) = TerminalsGuard.get(&TerminalId) {
		let TerminalStateGuard = TerminalArc.lock().unwrap();
		if let Some(Sender) = &TerminalStateGuard.PtyInputTransmitter {
			Sender
				.send(Text)
				.await
				.map_err(|e| CommonError::IpcError { Description:e.to_string() })?;
		}
	} else {
		return Err(CommonError::IpcError { Description:format!("Terminal with ID {} not found.", TerminalId) });
	}
	Ok(())
}

// Logic to dispose of a terminal instance.
pub async fn DisposeTerminalLogic<R:RunTime>(ApplicationHandle:&ApplicationHandle<R>, TerminalId:u64) -> Result<(), CommonError> {
	info!("[TerminalLogic] Disposing terminal ID: {}", TerminalId);
	let AppStateInstance = ApplicationHandle.state::<ApplicationState>();
	if let Some(TerminalArc) = AppStateInstance.ActiveTerminals.lock().unwrap().remove(&TerminalId) {
		let mut TerminalStateGuard = TerminalArc.lock().unwrap();
		// Abort the background tasks associated with this terminal.
		if let Some(handle_arc) = TerminalStateGuard.ReaderTaskHandle.take() {
			if let Some(handle) = handle_arc.lock().await.take() {
				handle.abort();
			}
		}
		if let Some(handle_arc) = TerminalStateGuard.ProcessWaitHandle.take() {
			if let Some(handle) = handle_arc.lock().await.take() {
				handle.abort();
			}
		}
	}
	Ok(())
}
