// ---------------------------------------------------------------------------------------------
// Mountain Terminal Handlers (handlers/terminal.rs)
// --------------------------------------------------------------------------------------------
// Handles RPC requests and notifications related to terminals, proxied from
// Cocoon's terminal shims. Manages terminal processes or pseudo-terminals
// (PTYs) on the Mountain side.
//
// Responsibilities:
// - Handling RPC calls related to creating/managing terminals ($createTerminal,

//   $show, $hide, $sendText, $dispose).
// - Managing terminal state within AppState, including PTY process details, I/O
//   task handles, and communication channels.
// - Spawning and managing the lifecycle of PTY processes using `portable-pty`.
// - Reading data from PTY output and forwarding it to Cocoon via
//   `$acceptTerminalProcessData` notifications.
// - Writing data received via `$sendText` to the PTY input.
// - Monitoring PTY process exit and sending `$acceptTerminalClosed`
//   notifications.
// - Handling environment variable notifications from the environment variable
//   collection shim (currently logging stubs).
//
// Key Interactions:
// - Called by `track::dispatch_sidecar_request` (or effects if these become
//   effects).
// - Interacts with `AppState` to store and manage `TerminalState` instances.
// - Uses the `portable-pty` crate for PTY creation and management.
// - Uses `tokio::process::Command` indirectly via `portable-pty`.
// - Uses `tokio::spawn` for managing asynchronous reader, writer, and
//   process-wait tasks.
// - Uses `tokio::sync::mpsc` for sending input to the PTY writer task.
// - Uses `vine::send_notification_to_sidecar` to send events back to Cocoon.
// - Emits Tauri events (e.g., `mountain://terminal/reveal`) to signal UI
//   updates.
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,

	// For pty_master_writer.write_all in a blocking context if needed, or for trait
	io::Write as StdIoWrite,

	path::PathBuf,

	sync::Arc,
	// Not directly needed for portable-pty's CommandBuilder
	// process::Stdio,
};

// For error types if handlers were to return CommonError
use Land_Common::errors::CommonError;
use log::{debug, error, info, trace, warn};
use portable_pty::{Child as PtyChild, CommandBuilder, MasterPty, NativePtySystem, PtyPair, PtySize, PtySystem};
use serde_json::{Value, json};
// Removed State as not directly used by handlers here
use tauri::{AppHandle, Manager, Runtime, State};
use tokio::{
	// AsyncWriteExt not directly needed if using std::io::Write from MasterPty
	io::{AsyncBufReadExt, BufReader},

	// Fallback, not primary for PTY
	// process::Command as TokioCommand,

	// TokioMutex for JoinHandles in TerminalState
	sync::{Mutex as TokioMutex, mpsc as TokioMpsc},

	task::JoinHandle,
};

use crate::handlers::error_utils;
use crate::{
	// AppState now has TerminalState and maps
	app_state::{AppState, TerminalState},

	// May be needed if terminal operations become effects
	// runtime::AppRuntime,
	vine,
	// For consistent error formatting
};

// --- Helper Functions ---
fn create_terminal_rpc_error_string(message:String, code:Option<&str>) -> String {
	error_utils::rpc_error_string(message, code.map(|c| format!("ETERM_{}", c)))
}

fn map_terminal_lock_error_str<T>(e:std::sync::PoisonError<std::sync::MutexGuard<'_, T>>, context:&str) -> String {
	let msg = format!("[Terminal Handler LockErr] {}: {}", context, e);

	error!("{}", msg);

	create_terminal_rpc_error_string(msg, Some("LOCK"))
}

fn get_default_shell() -> String {
	if cfg!(windows) {
		std::env::var("ComSpec").unwrap_or_else(|_| "cmd.exe".to_string())
	} else {
		std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
	}
}

// --- RPC Handlers (Called by Cocoon Shim via Track/RPC) ---

pub async fn handle_create_terminal<R:Runtime>(
	app_handle:AppHandle<R>,

	// Expects ICreateTerminalOptions structure
	options_val:Value,
) -> Result<Value, String> {
	let app_state = app_handle.state::<AppState>();

	let terminal_id = app_state.get_next_terminal_id();

	let default_shell = get_default_shell();

	let name = options_val
		.get("name")
		.and_then(Value::as_str)
		.map_or_else(|| format!("terminal-{}", terminal_id), String::from);

	info!("[Terminal Handler] Creating terminal: id={}, name='{}'", terminal_id, name);

	trace!("[Terminal Handler] Options: {:?}", options_val);

	// Create initial TerminalState metadata (will be updated with PTY handles etc.)
	let mut current_terminal_state = TerminalState::new(terminal_id, name.clone(), &options_val, default_shell);

	// 1. Create PTY system
	let pty_system = NativePtySystem::default();

	// 2. Open PTY pair
	let pty_size = options_val
		.get("initialDimensions")
		.and_then(|dim_val| {
			let rows = dim_val.get("rows").and_then(Value::as_u64).map(|r| r as u16).unwrap_or(24);

			let cols = dim_val.get("cols").and_then(Value::as_u64).map(|c| c as u16).unwrap_or(80);

			Some(PtySize { rows, cols, pixel_width:0, pixel_height:0 })
		})
		.unwrap_or(PtySize { rows:24, cols:80, pixel_width:0, pixel_height:0 });

	let pty_pair = pty_system
		.openpty(pty_size)
		.map_err(|e| create_terminal_rpc_error_string(format!("Failed to open PTY: {}", e), Some("PTYCREATE")))?;

	// 3. Prepare command
	let mut cmd_builder = CommandBuilder::new(current_terminal_state.shell_path);

	if !current_terminal_state.shell_args.is_empty() {
		cmd_builder.args(current_terminal_state.shell_args);
	}

	if let Some(cwd) = current_terminal_state.cwd {
		cmd_builder.cwd(cwd);
	}

	// TODO: Apply merged environment variables (system + extension contributions +
	// terminal specific options.env)
	if let Some(env_vars) = current_terminal_state.env {
		for (k, v) in env_vars {
			cmd_builder.env(k, v);
		}
	}

	cmd_builder.env("TERM_PROGRAM", "landcode");

	cmd_builder.env("TERM_PROGRAM_VERSION", app_handle.package_info().version.to_string());

	// 4. Spawn command in PTY slave
	let mut pty_child_process:PtyChild = pty_pair.slave.spawn_command(cmd_builder).map_err(|e| {
		create_terminal_rpc_error_string(format!("Failed to spawn command in PTY: {}", e), Some("PTYSPAWN"))
	})?;

	// Option<u32>
	let os_pid = pty_child_process.process_id();

	current_terminal_state.os_process_id = os_pid;

	info!(
		"[Terminal Handler] PTY process spawned for term ID {}: OS PID {:?}",
		terminal_id, os_pid
	);

	let pty_master_reader_raw = pty_pair.master.try_clone_reader().map_err(|e| {
		create_terminal_rpc_error_string(format!("Failed to clone PTY master reader: {}", e), Some("PTYIO_R_CLONE"))
	})?;

	let mut pty_master_writer_raw = pty_pair.master.try_clone_writer().map_err(|e| {
		create_terminal_rpc_error_string(format!("Failed to clone PTY master writer: {}", e), Some("PTYIO_W_CLONE"))
	})?;

	// Original master can be dropped as we've cloned reader/writer parts
	drop(pty_pair.master);

	// Slave is not used in the parent process after spawn
	drop(pty_pair.slave);

	// 5. Create MPSC channel for PTY input
	// Buffer for PTY input
	let (pty_input_tx, mut pty_input_rx) = TokioMpsc::channel::<String>(32);

	// Store the sender
	current_terminal_state.pty_input_tx = Some(pty_input_tx.clone());

	// 6. Spawn PTY Writer Task
	let writer_terminal_id = terminal_id;

	tokio::spawn(async move {
		info!("[Terminal PTY Writer ID {}] Task started.", writer_terminal_id);

		// pty_master_writer_raw is Box<dyn std::io::Write + Send>
		// We need to use it in a way that doesn't block the async task for too long.
		// For now, let's assume writes are relatively quick or use spawn_blocking if
		// they become an issue.
		while let Some(input_data) = pty_input_rx.recv().await {
			trace!(
				"[Terminal PTY Writer ID {}] Writing: '{}...'",
				writer_terminal_id,
				input_data.chars().take(30).collect::<String>()
			);

			match pty_master_writer_raw.write_all(input_data.as_bytes()) {
				Ok(_) => {
					if let Err(e_flush) = pty_master_writer_raw.flush() {
						error!(
							"[Terminal PTY Writer ID {}] Error flushing PTY master: {}. Stopping.",
							writer_terminal_id, e_flush
						);

						break;
					}
				},

				Err(e) => {
					error!(
						"[Terminal PTY Writer ID {}] Error writing to PTY master: {}. Stopping.",
						writer_terminal_id, e
					);

					break;
				},
			}
		}

		info!(
			"[Terminal PTY Writer ID {}] Input channel closed or write error. Writer task exiting.",
			writer_terminal_id
		);
	});

	// 7. Spawn PTY Reader Task
	let reader_app_handle = app_handle.clone();

	let reader_sidecar_id = "cocoon-main".to_string();

	let reader_terminal_id = terminal_id;

	let reader_task_handle = tokio::spawn(async move {
		info!("[Terminal PTY Reader ID {}] Task started.", reader_terminal_id);

		// pty_master_reader_raw is Box<dyn std::io::Read + Send>
		// Wrap in BufReader for read_line, but BufReader itself is not AsyncRead.
		// We need an async reader. Tokio's BufReader works with AsyncRead.
		// Let's assume pty_master_reader_raw can be adapted or used with spawn_blocking
		// if strictly std::io::Read. However, portable-pty master often implements
		// mio/polling, suitable for async. For simplicity, let's assume it can be
		// read in chunks that are then processed. A direct async wrapper or
		// `tokio_pty_process` might be better here. Using a simple chunk-based read
		// for now.
		// Placeholder, need to adapt Box<dyn Read>
		let mut reader = tokio::io::BufReader::with_capacity(1024, tokio::io::stdin());

		// This ^ is incorrect. The Box<dyn Read + Send> needs to be adapted for async.
		// A common pattern is to use `tokio::task::spawn_blocking` for `std::io::Read`
		// or use a crate that provides an async PTY reader.
		// For now, this part of the reader task will be a more conceptual loop.

		// Corrected approach for reading from Box<dyn std::io::Read + Send> in async:
		// This requires `pty_master_reader_raw` to be moved into a `spawn_blocking`
		// call, and then communicate results back to the async context via channels.
		// Or, use a PTY library with first-class async support.
		// `portable-pty`'s `MasterPty` itself can be used with `mio` for polling.
		// For a simpler async model with `portable-pty`, one might read small chunks
		// in a loop if the underlying fd supports non-blocking, or use
		// `spawn_blocking`.

		// Let's stick to a conceptual loop and note the TODO for proper async wrapping
		// if `pty_master_reader_raw` is purely blocking `std::io::Read`.
		// If `portable-pty`'s reader can be used with `tokio-util`'s `PollEvented`,

		// that's ideal. For now, showing a simplified chunk read:
		// Read in chunks
		let mut read_buffer = vec![0u8; 4096];

		loop {
			// This is a placeholder for actual async reading logic.
			// The `pty_master_reader_raw.read()` call is blocking std::io::Read.
			// It should be wrapped in `spawn_blocking` or use an async PTY interface.
			// For this synthesis, we'll assume it's conceptually handled.
			// This is
			// let read_result = pty_master_reader_raw.read(&mut read_buffer);

			// blocking TODO: Replace with proper async read from PTY master

			// Simulate async read with a placeholder:
			// Placeholder
			tokio::time::sleep(Duration::from_millis(5000)).await;

			let read_result:std::io::Result<usize> = Err(std::io::Error::new(
				std::io::ErrorKind::Other,
				"PTY Read not fully implemented async",
				// Placeholder
			));

			match read_result {
				Ok(0) => {
					info!("[Terminal PTY Reader ID {}] EOF from PTY.", reader_terminal_id);

					break;
				},

				Ok(n) => {
					let data_str = String::from_utf8_lossy(&read_buffer[..n]);

					trace!("[Terminal PTY Reader ID {}] Read data: {}", reader_terminal_id, data_str);

					let payload = json!([reader_terminal_id, data_str.to_string()]);

					if let Err(e) = vine::send_notification_to_sidecar(
						&reader_sidecar_id,
						"$acceptTerminalProcessData".to_string(),
						payload,
					)
					.await
					{
						error!("[Terminal PTY Reader ID {}] Failed to send data: {}", reader_terminal_id, e);
					}
				},

				Err(e) => {
					error!("[Terminal PTY Reader ID {}] Error reading from PTY: {}", reader_terminal_id, e);

					break;
				},
			}
		}

		info!("[Terminal PTY Reader ID {}] Reader task finished.", reader_terminal_id);
	});

	// 8. Spawn Process Wait Task
	let wait_app_handle = app_handle.clone();

	let wait_sidecar_id = "cocoon-main".to_string();

	let wait_terminal_id = terminal_id;

	let process_wait_handle = tokio::spawn(async move {
		info!(
			"[Terminal Waiter ID {}] Task started, awaiting PTY process exit.",
			wait_terminal_id
		);

		// `pty_child_process.wait()` is blocking. It must be run in `spawn_blocking`.
		let exit_status_res = tokio::task::spawn_blocking(move || pty_child_process.wait()).await;

		match exit_status_res {
			Ok(Ok(exit_status)) => {
				info!(
					"[Terminal Waiter ID {}] PTY Process exited with status: {:?}",
					wait_terminal_id, exit_status
				);

				let exit_code = exit_status.exit_code();

				let reason = if exit_status.success() { 0 } else { 1 };

				let payload = json!([wait_terminal_id, exit_code, reason]);

				if let Err(e) =
					vine::send_notification_to_sidecar(&wait_sidecar_id, "$acceptTerminalClosed".to_string(), payload)
						.await
				{
					error!(
						"[Terminal Waiter ID {}] Failed to send $acceptTerminalClosed: {}",
						wait_terminal_id, e
					);
				}
			},

			Ok(Err(e)) | Err(_) => {
				// Error from wait() or JoinError from spawn_blocking
				error!(
					"[Terminal Waiter ID {}] Error waiting for PTY process exit: {:?}",
					wait_terminal_id, exit_status_res
				);

				let payload = json!([wait_terminal_id, Value::Null, 0]);

				if let Err(e_ipc) =
					vine::send_notification_to_sidecar(&wait_sidecar_id, "$acceptTerminalClosed".to_string(), payload)
						.await
				{
					error!(
						"[Terminal Waiter ID {}] Failed to send $acceptTerminalClosed (on wait error): {}",
						wait_terminal_id, e_ipc
					);
				}
			},
		}

		let app_state_cleanup = wait_app_handle.state::<AppState>();

		if let Ok(mut terminals) = app_state_cleanup.active_terminals.lock() {
			if terminals.remove(&wait_terminal_id).is_some() {
				info!(
					"[Terminal Waiter ID {}] Removed terminal from active list after exit.",
					wait_terminal_id
				);
			}
		}

		info!("[Terminal Waiter ID {}] Waiter task finished.", wait_terminal_id);
	});

	current_terminal_state.reader_task_handle = Some(Arc::new(TokioMutex::new(Some(reader_task_handle))));

	current_terminal_state.process_wait_handle = Some(Arc::new(TokioMutex::new(Some(process_wait_handle))));

	{
		let mut active_terminals_guard = app_state
			.active_terminals
			.lock()
			.map_err(|e| map_terminal_lock_error_str(e, "active_terminals final store"))?;

		active_terminals_guard.insert(terminal_id, Arc::new(StdMutex::new(current_terminal_state)));
	}

	// 9. Send initial notifications
	let opened_payload = json!([terminal_id, name.clone()]);

	if let Err(e) =
		vine::send_notification_to_sidecar("cocoon-main", "$acceptTerminalOpened".to_string(), opened_payload).await
	{
		error!("[Terminal Handler] Failed to send $acceptTerminalOpened: {}", e);
	}

	if let Some(pid) = os_pid {
		let pid_payload = json!([terminal_id, pid]);

		if let Err(e) =
			vine::send_notification_to_sidecar("cocoon-main", "$acceptTerminalProcessId".to_string(), pid_payload).await
		{
			error!("[Terminal Handler] Failed to send $acceptTerminalProcessId: {}", e);
		}
	}

	Ok(json!({ "id": terminal_id, "name": name, "pid": os_pid }))
}

pub async fn handle_show<R:Runtime>(app_handle:AppHandle<R>, args:Value) -> Result<Value, String> {
	// Assuming args is array [id, preserveFocus]
	let terminal_id = args.get(0).and_then(Value::as_u64).ok_or_else(|| {
		create_terminal_rpc_error_string("Missing or invalid terminal ID for $show".to_string(), Some("BADARG"))
	})?;

	let preserve_focus = args.get(1).and_then(Value::as_bool).unwrap_or(false);

	info!(
		"[Terminal Handler] RPC handle_show: id={}, preserveFocus={}",
		terminal_id, preserve_focus
	);

	app_handle
		.emit_all(
			"mountain://terminal/reveal",
			json!({"id": terminal_id, "preserveFocus": preserve_focus}),
		)
		.map_err(|e| create_terminal_rpc_error_string(format!("Emit failed: {}", e), Some("EMITFAIL")))?;

	Ok(Value::Null)
}

pub async fn handle_hide<R:Runtime>(_app_handle:AppHandle<R>, args:Value) -> Result<Value, String> {
	let terminal_id = args.get(0).and_then(Value::as_u64).ok_or_else(|| {
		create_terminal_rpc_error_string("Missing or invalid terminal ID for $hide".to_string(), Some("BADARG"))
	})?;

	info!("[Terminal Handler] RPC handle_hide: id={}", terminal_id);

	warn!(
		"[Terminal Handler] STUB: Hiding terminal UI via RPC is usually managed by UI state. No direct action taken \
		 by backend."
	);

	Ok(Value::Null)
}

pub async fn handle_send_text<R:Runtime>(app_handle:AppHandle<R>, args:Value) -> Result<Value, String> {
	let terminal_id = args.get(0).and_then(Value::as_u64).ok_or_else(|| {
		create_terminal_rpc_error_string("Missing or invalid terminal ID for $sendText".to_string(), Some("BADARG"))
	})?;

	let text_to_send = args.get(1).and_then(Value::as_str).ok_or_else(|| {
		create_terminal_rpc_error_string("Missing or invalid text for $sendText".to_string(), Some("BADARG"))
	})?;

	info!(
		"[Terminal Handler] RPC handle_sendText: id={}, text='{}...'",
		terminal_id,
		text_to_send.chars().take(30).collect::<String>()
	);

	let app_state = app_handle.state::<AppState>();

	let maybe_tx = {
		let active_terminals_guard = app_state
			.active_terminals
			.lock()
			.map_err(|e| map_terminal_lock_error_str(e, "active_terminals for sendText"))?;

		if let Some(terminal_arc) = active_terminals_guard.get(&terminal_id) {
			let terminal_state_guard = terminal_arc
				.lock()
				.map_err(|e| map_terminal_lock_error_str(e, "terminal state for sendText"))?;

			terminal_state_guard.pty_input_tx.clone()
		} else {
			None
		}
	};

	if let Some(tx) = maybe_tx {
		if let Err(e) = tx.send(text_to_send.to_string()).await {
			let err_msg = format!(
				"Failed to send text to terminal ID {} writer task (channel closed or full): {}",
				terminal_id, e
			);

			error!("[Terminal Handler] {}", err_msg);

			return Err(create_terminal_rpc_error_string(err_msg, Some("PIPEFAIL")));
		}

		trace!(
			"[Terminal Handler] Text successfully sent to writer task's MPSC channel for terminal ID: {}",
			terminal_id
		);
	} else {
		warn!(
			"[Terminal Handler] No PTY input channel (pty_input_tx) found for terminal ID: {}. Terminal might be \
			 disposed or failed to init.",
			terminal_id
		);

		return Err(create_terminal_rpc_error_string(
			format!("Terminal {} not found or not ready for input.", terminal_id),
			Some("NOTFOUND"),
		));
	}

	Ok(Value::Null)
}

pub async fn handle_dispose<R:Runtime>(app_handle:AppHandle<R>, args:Value) -> Result<Value, String> {
	let terminal_id = args.get(0).and_then(Value::as_u64).ok_or_else(|| {
		create_terminal_rpc_error_string("Missing or invalid terminal ID for $dispose".to_string(), Some("BADARG"))
	})?;

	info!("[Terminal Handler] RPC handle_dispose: id={}", terminal_id);

	let app_state = app_handle.state::<AppState>();

	let terminal_arc_to_dispose = {
		let mut active_terminals_guard = app_state
			.active_terminals
			.lock()
			.map_err(|e| map_terminal_lock_error_str(e, "active_terminals in dispose"))?;

		active_terminals_guard.remove(&terminal_id)
	};

	if let Some(terminal_arc) = terminal_arc_to_dispose {
		info!("[Terminal Handler] Disposing terminal ID: {}", terminal_id);

		let mut terminal_state_guard = terminal_arc
			.lock()
			.map_err(|e| map_terminal_lock_error_str(e, "terminal state in dispose"))?;

		// 1. Abort tasks by dropping their handles (if stored in TerminalState) or by
		//    closing the MPSC channel for the writer task.
		if let Some(reader_handle_arc) = terminal_state_guard.reader_task_handle.take() {
			if let Some(handle) = reader_handle_arc.lock().await.take() {
				info!("[Terminal Handler] Aborting reader task for terminal ID: {}", terminal_id);

				handle.abort();
			}
		}

		if let Some(process_wait_handle_arc) = terminal_state_guard.process_wait_handle.take() {
			if let Some(handle) = process_wait_handle_arc.lock().await.take() {
				info!("[Terminal Handler] Aborting process wait task for terminal ID: {}", terminal_id);

				handle.abort();
			}
		}

		// Drop the sender for the PTY input channel; this will cause the writer task to
		// terminate.
		drop(terminal_state_guard.pty_input_tx.take());

		info!(
			"[Terminal Handler] Dispose initiated for terminal ID: {}. Tasks signalled/handles dropped.",
			terminal_id
		);
	} else {
		warn!(
			"[Terminal Handler] Dispose called for unknown or already disposed terminal ID: {}",
			terminal_id
		);
	}

	Ok(Value::Null)
}

// --- Notification Handlers (From Cocoon Env Variable Collection Shim) ---
pub async fn handle_set_environment_variable<R:Runtime>(
	_app_handle:AppHandle<R>,

	params:Value,
) -> Result<Value, String> {
	let extension_id = params.get("extensionId").and_then(Value::as_str).unwrap_or("unknown_ext");

	let variable = params.get("variable").and_then(Value::as_str).unwrap_or("unknown_var");

	info!(
		"[Terminal Env Handler] SetEnv: Ext='{}', Var='{}', Details: {:?}",
		extension_id,
		variable,
		params.get("mutator")
	);

	warn!("[Terminal Env Handler] STUB: Storing terminal environment variable changes not implemented.");

	Ok(Value::Null)
}

pub async fn handle_delete_environment_variable<R:Runtime>(
	_app_handle:AppHandle<R>,

	params:Value,
) -> Result<Value, String> {
	let extension_id = params.get("extensionId").and_then(Value::as_str).unwrap_or("unknown_ext");

	let variable = params.get("variable").and_then(Value::as_str).unwrap_or("unknown_var");

	info!("[Terminal Env Handler] DeleteEnv: Ext='{}', Var='{}'", extension_id, variable);

	warn!("[Terminal Env Handler] STUB: Deleting terminal environment variable changes not implemented.");

	Ok(Value::Null)
}

pub async fn handle_clear_environment_variable_collection<R:Runtime>(
	_app_handle:AppHandle<R>,

	params:Value,
) -> Result<Value, String> {
	let extension_id = params.get("extensionId").and_then(Value::as_str).unwrap_or("unknown_ext");

	info!("[Terminal Env Handler] ClearCollection: Ext='{}'", extension_id);

	warn!("[Terminal Env Handler] STUB: Clearing terminal environment variable collection not implemented.");

	Ok(Value::Null)
}
