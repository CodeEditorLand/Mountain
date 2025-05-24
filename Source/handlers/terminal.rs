// ---------------------------------------------------------------------------------------------
// Mountain Terminal Handlers (handlers/terminal.rs)
// --------------------------------------------------------------------------------------------
// Handles RPC requests and notifications related to integrated terminals,

// proxied from Cocoon's terminal shims (`extHostTerminalService.ts`,

// `extHostTerminalProcess.ts`). It manages terminal processes or
// pseudo-terminals (PTYs) on the Mountain (native) side.
//
// Responsibilities:
// - Handling RPC calls for terminal management:
//   - `$createTerminal`: Creates a new terminal instance, spawns a PTY with the
//     specified shell and options.
//   - `$show`: Requests the frontend (Sky) to reveal a specific terminal.
//   - `$hide`: (Largely a no-op in backend) Informs that a terminal view might
//     be hidden by UI.
//   - `$sendText`: Writes input text from the extension to the PTY of a
//     specific terminal.
//   - `$dispose`: Terminates the PTY process and cleans up resources for a
//     terminal.
// - Managing terminal state (`TerminalState`) within
//   `AppState.active_terminals`, including PTY process details, I/O task
//   handles, and communication channels.
// - Spawning and managing the lifecycle of PTY processes using the
//   `portable-pty` crate.
// - Asynchronously reading data from PTY output (stdout) and forwarding it to
//   Cocoon via `$acceptTerminalProcessData` Vine notifications.
// - Asynchronously writing data received from Cocoon (via `$sendText`) to the
//   PTY input (stdin) using an MPSC channel.
// - Monitoring PTY process exit and sending `$acceptTerminalClosed` Vine
//   notifications to Cocoon.
// - Handling environment variable modification notifications from Cocoon's
//   environment variable collection shim (currently implemented as logging
//   stubs).
//
// Key Interactions:
// - Called by `track::dispatch_sidecar_request` (or `rpc.rs`) for terminal
//   RPCs.
// - Interacts with `AppState` to store and manage `TerminalState` instances in
//   `active_terminals` and to get unique terminal IDs.
// - Uses the `portable-pty` crate for cross-platform PTY creation and
//   management.
// - Uses `tokio::process::Command` indirectly via `portable-pty`'s
//   `CommandBuilder`.
// - Uses `tokio::spawn` for managing asynchronous PTY reader, PTY writer, and
//   process-wait tasks for each terminal.
// - Uses `tokio::sync::mpsc` for sending input data from `$sendText` to the
//   dedicated PTY writer task.
// - Uses `vine::send_notification_to_sidecar` to send events (data, close, PID)
//   back to Cocoon.
// - Emits Tauri events (e.g., `mountain://terminal/reveal`) to signal UI
//   updates to Sky.
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,

	// For `pty_master_writer.flush()`
	io::Write as StdIoWrite,

	path::PathBuf,

	sync::Arc,
	// `process::Stdio` from `std` is not directly needed as `portable-pty` handles PTY setup.
};

// `CommonError` might be used if these handlers were part of an effect system.
// Currently, they return `Result<Value, String>` for RPC.
// use Land_Common::errors::CommonError;
use log::{debug, error, info, trace, warn};
use portable_pty::{Child as PtyChild, CommandBuilder, MasterPty, NativePtySystem, PtyPair, PtySize, PtySystem};
use serde_json::{Value, json};
// `State` is not directly used in handler signatures here.
use tauri::{AppHandle, Manager, Runtime};
use tokio::{
	// `AsyncWriteExt` not directly needed for `MasterPty`'s writer if using its `Write` trait.
	// AsyncReadExt for read_exact or read_buf
	io::{AsyncBufReadExt, AsyncReadExt, BufReader},

	sync::{Mutex as TokioMutex, mpsc as TokioMpsc},

	task::JoinHandle,

	// For sleep in placeholder reader
	time::Duration as TokioDuration,
};

// For consistent RPC error formatting
use crate::handlers::error_utils;
use crate::{
	// AppState stores TerminalState and maps
	app_state::{AppState, TerminalState},

	// For sending notifications to Cocoon
	vine,
};

// --- Helper Functions ---

/// Creates a JSON-RPC error string specific to terminal operations.
///
/// Prefixes the error code with "ETERM_" (e.g., "ETERM_PTYCREATE").
///
/// # Arguments
/// * `message` - The error message.
/// * `code` - An optional short code (e.g., "PTYCREATE", "LOCK").
///
/// # Returns
/// A `String` containing the JSON-formatted RPC error.
fn create_terminal_operation_rpc_error(message:String, code:Option<&str>) -> String {
	let full_code = code.map_or_else(
		// Default if no specific code part
		|| "ETERM_UNKNOWN".to_string(),
		|c| format!("ETERM_{}", c.to_uppercase()),
	);

	error_utils::rpc_error_string(message, Some(&full_code))
}

/// Formats a `PoisonError` from a Mutex lock on terminal-related `AppState`
/// sections into a standardized RPC error string.
///
/// # Arguments
/// * `e` - The `PoisonError`.
/// * `context` - A string describing the context of the lock (e.g.,
///
///   "active_terminals map").
///
/// # Returns
/// A `String` containing a JSON-formatted RPC error.
fn format_terminal_app_state_lock_error_for_rpc<T>(
	e:std::sync::PoisonError<std::sync::MutexGuard<'_, T>>,

	context:&str,
) -> String {
	let msg = format!("[Terminal Handler LockErr] Failed to acquire lock on {}: {}", context, e);

	// Log detailed internal error
	error!("{}", msg);

	create_terminal_operation_rpc_error(msg, Some("LOCK_STATE"))
}

/// Determines the default shell executable path for the current operating
/// system.
///
/// - Windows: Uses `ComSpec` environment variable (typically `cmd.exe`),
///
///   fallback to `powershell.exe` or `cmd.exe`.
/// - Other (Unix-like): Uses `SHELL` environment variable (e.g., `/bin/bash`,
///
///   `/bin/zsh`), fallback to `/bin/sh`.
///
/// # Returns
/// A `String` with the path to the default shell.
fn get_platform_default_shell_path() -> String {
	if cfg!(windows) {
		std::env::var("ComSpec").unwrap_or_else(|_| {
			// Fallback logic for Windows if ComSpec is not set
			// Prefer PowerShell if available, then cmd.exe
			// This is a basic check; a more robust solution might involve checking PATH.
			// For now, hardcoding common fallbacks.
			// TODO: Add more robust Windows default shell detection (e.g., checking if
			// PowerShell is installed).
			warn!(
				"[Terminal DefaultShell] ComSpec environment variable not found. Falling back to 'powershell.exe' or \
				 'cmd.exe'."
			);

			// Or "cmd.exe"
			"powershell.exe".to_string()
		})
	} else {
		std::env::var("SHELL").unwrap_or_else(|_| {
			warn!("[Terminal DefaultShell] SHELL environment variable not found. Falling back to '/bin/sh'.");

			"/bin/sh".to_string()
		})
	}
}

// --- RPC Handlers (Called by Cocoon Shim via Track/RPC) ---

/// Handles the `$createTerminal` RPC call from Cocoon.
///
/// Creates a new pseudo-terminal (PTY), spawns the specified shell (or default)
/// within it, and sets up asynchronous tasks for reading PTY output and writing
/// PTY input. Stores the `TerminalState` in `AppState`.
///
/// # Arguments
/// * `app_handle` - The Tauri `AppHandle`.
/// * `options_val` - A `serde_json::Value` representing
///   `vscode.TerminalOptions` (or `ICreateTerminalOptions` from Cocoon's shim).
///   Expected fields include `name`, `shellPath`, `shellArgs`, `cwd`, `env`,
///
///   `initialDimensions`, `isPty`.
///
/// # Returns
/// * `Ok(Value)`: A JSON object `{ "id": u64, "name": string, "pid":
///   Option<u32> }` with the new terminal's ID, name, and OS process ID.
/// * `Err(String)`: A JSON-RPC error string if PTY creation or process spawning
///   fails.
pub async fn handle_create_terminal<R:Runtime>(
	app_handle:AppHandle<R>,

	// Expects ICreateTerminalOptions structure
	options_val:Value,
) -> Result<Value, String> {
	let app_state = app_handle.state::<AppState>();

	// Atomically get a new ID
	let terminal_id = app_state.get_next_terminal_id();

	let default_shell_path = get_platform_default_shell_path();

	let name_from_options = options_val.get("name").and_then(Value::as_str);

	let terminal_name = name_from_options.map_or_else(|| format!("terminal-{}", terminal_id), String::from);

	info!(
		"[Terminal Handler Create] Attempting to create terminal: id={}, name='{}'",
		terminal_id, terminal_name
	);

	trace!("[Terminal Handler Create] Options received: {:?}", options_val);

	// Create initial TerminalState metadata. This will be updated with PTY handles,

	// task JoinHandles, etc., as they are created.
	let mut current_terminal_state =
		TerminalState::new(terminal_id, terminal_name.clone(), &options_val, default_shell_path);

	// 1. Create PTY system instance.
	let pty_system = NativePtySystem::default();

	// 2. Determine PTY size from options or use defaults.
	let pty_initial_size = options_val
		.get("initialDimensions")
		.and_then(|dim_val| {
			// Default rows
			let rows = dim_val.get("rows").and_then(Value::as_u64).map(|r| r as u16).unwrap_or(24);

			// Default columns
			let cols = dim_val.get("cols").and_then(Value::as_u64).map(|c| c as u16).unwrap_or(80);

			// pixel_width and pixel_height are often 0 unless specific font metrics are
			// known.
			Some(PtySize { rows, cols, pixel_width:0, pixel_height:0 })
		})
		.unwrap_or(PtySize { rows:24, cols:80, pixel_width:0, pixel_height:0 });

	// 3. Open a new PTY pair (master and slave).
	let pty_pair = pty_system
		.openpty(pty_initial_size)
		.map_err(|e| create_terminal_operation_rpc_error(format!("Failed to open PTY pair: {}", e), Some("PTYOPEN")))?;

	// 4. Prepare the command to run in the PTY slave.
	let mut cmd_builder = CommandBuilder::new(&current_terminal_state.shell_path);

	if !current_terminal_state.shell_args.is_empty() {
		cmd_builder.args(&current_terminal_state.shell_args);
	}

	if let Some(cwd_path) = &current_terminal_state.cwd {
		cmd_builder.cwd(cwd_path);
	}

	// Apply custom environment variables from options, merged with system
	// environment. TODO: Implement merging with system environment and
	// extension-contributed environment variables.       For now,

	// `cmd_builder.env` adds/overrides.
	if let Some(env_vars_map) = &current_terminal_state.env {
		for (key, value) in env_vars_map {
			cmd_builder.env(key, value);
		}
	}

	// Set standard TERM_PROGRAM variables, useful for shells/apps running in
	// terminal.
	// Consistent with VS Code
	cmd_builder.env("TERM_PROGRAM", "landcode");

	cmd_builder.env("TERM_PROGRAM_VERSION", app_handle.package_info().version.to_string());

	// 5. Spawn the command in the PTY slave.
	let mut pty_child_process:PtyChild = pty_pair.slave.spawn_command(cmd_builder).map_err(|e| {
		create_terminal_operation_rpc_error(
			format!(
				"Failed to spawn command '{}' in PTY slave: {}",
				current_terminal_state.shell_path, e
			),
			Some("PTYSPAWN"),
		)
	})?;

	// Option<u32>
	current_terminal_state.os_process_id = pty_child_process.process_id();

	info!(
		"[Terminal Handler Create] PTY process spawned for term ID {}: OS PID {:?}, Shell: '{}'",
		terminal_id, current_terminal_state.os_process_id, current_terminal_state.shell_path
	);

	// Clone reader/writer parts from the PTY master.
	// These are `Box<dyn Read + Send>` and `Box<dyn Write + Send>`.
	let pty_master_reader_handle = pty_pair.master.try_clone_reader().map_err(|e| {
		create_terminal_operation_rpc_error(format!("Failed to clone PTY master reader: {}", e), Some("PTYIO_R_CLONE"))
	})?;

	let mut pty_master_writer_handle = pty_pair.master.try_clone_writer().map_err(|e| {
		create_terminal_operation_rpc_error(format!("Failed to clone PTY master writer: {}", e), Some("PTYIO_W_CLONE"))
	})?;

	// Original master and slave can be dropped as we have cloned parts.
	drop(pty_pair.master);

	// Slave is primarily for the child process.
	drop(pty_pair.slave);

	// 6. Create MPSC channel for sending input from Mountain to the PTY writer
	//    task.
	// Buffer size for PTY input
	let (pty_input_tx, mut pty_input_rx_for_writer_task) = TokioMpsc::channel::<String>(32);

	// Store sender in TerminalState
	current_terminal_state.pty_input_tx = Some(pty_input_tx.clone());

	// 7. Spawn PTY Writer Task (writes data from pty_input_rx to PTY master input).
	let writer_task_terminal_id = terminal_id;

	let writer_task_handle = tokio::spawn(async move {
		info!("[Terminal PTY Writer][ID {}] Writer task started.", writer_task_terminal_id);

		// `pty_master_writer_handle` implements `std::io::Write`.
		// Writes should be reasonably quick. If they block excessively,

		// `tokio::task::spawn_blocking` might be needed for each write.
		while let Some(input_data_str) = pty_input_rx_for_writer_task.recv().await {
			trace!(
				"[Terminal PTY Writer][ID {}] Received text to write (len {}): '{}...'",
				writer_task_terminal_id,
				input_data_str.len(),
				input_data_str.chars().take(30).collect::<String>()
			);

			match pty_master_writer_handle.write_all(input_data_str.as_bytes()) {
				Ok(_) => {
					// Flushing is important to ensure data reaches the PTY process.
					if let Err(e_flush) = pty_master_writer_handle.flush() {
						error!(
							"[Terminal PTY Writer][ID {}] Error flushing PTY master input: {}. Stopping writer task.",
							writer_task_terminal_id, e_flush
						);

						// Exit task on flush error
						break;
					}
				},

				Err(e_write) => {
					error!(
						"[Terminal PTY Writer][ID {}] Error writing to PTY master input: {}. Stopping writer task.",
						writer_task_terminal_id, e_write
					);

					// Exit task on write error
					break;
				},
			}
		}

		info!(
			"[Terminal PTY Writer][ID {}] Input channel closed or write error. Writer task exiting.",
			writer_task_terminal_id
		);

		// Dropping `pty_master_writer_handle` here might close the PTY's input
		// from Mountain's side.
	});

	// TODO: Store `writer_task_handle` in `TerminalState` if explicit abortion is
	// needed, though it usually exits when `pty_input_rx_for_writer_task` channel
	// closes.

	// 8. Spawn PTY Reader Task (reads data from PTY master output and sends to
	//    Cocoon).
	let reader_task_app_handle = app_handle.clone();

	// TODO: Make configurable if needed
	let reader_task_sidecar_id = "cocoon-main".to_string();

	let reader_task_terminal_id = terminal_id;

	let reader_task_handle = tokio::spawn(async move {
		info!("[Terminal PTY Reader][ID {}] Reader task started.", reader_task_terminal_id);

		// `pty_master_reader_handle` is `Box<dyn std::io::Read + Send>`.
		// To use it asynchronously with Tokio, wrap it appropriately or use
		// `spawn_blocking`. `portable-pty` master often provides a file descriptor
		// that can be used with Mio/polling. A simpler approach for Tokio is to
		// `spawn_blocking` for each read operation if the Read impl is blocking.
		// However, for a continuous stream, a dedicated thread with `spawn_blocking`
		// that sends data back to this async task via an MPSC channel is more robust.

		// Simpler, potentially less efficient direct async read attempt:
		// This requires that the underlying Read implementation is compatible with
		// being wrapped by Tokio's async adapters or is itself non-blocking.
		// `tokio::io::util::Poll Međutim, Box<dyn Read> cannot be directly converted to
		// AsyncRead easily. The most robust way is `spawn_blocking` or using an
		// async-native PTY library if available. For this example, let's assume a
		// loop that uses `tokio::task::spawn_blocking` for reads. This is less ideal
		// than a true async PTY reader but works.

		// Create a channel to receive data from the blocking reader thread.
		let (data_tx, mut data_rx) = TokioMpsc::channel::<Result<Vec<u8>, std::io::Error>>(4);

		let blocking_reader_thread_handle = tokio::task::spawn_blocking(move || {
			// Moved into blocking task
			let mut pty_reader = pty_master_reader_handle;

			// Reusable buffer for reads
			let mut buffer = vec![0u8; 4096];

			loop {
				match pty_reader.read(&mut buffer) {
					Ok(0) => {
						// EOF
						info!(
							"[Terminal PTY Reader Blocking][ID {}] EOF from PTY master. Stopping blocking reader.",
							reader_task_terminal_id
						);

						// Signal EOF with empty vec
						let _ = data_tx.blocking_send(Ok(Vec::new()));

						break;
					},

					Ok(n) => {
						if data_tx.blocking_send(Ok(buffer[..n].to_vec())).is_err() {
							// Receiver dropped, async task likely terminated
							warn!(
								"[Terminal PTY Reader Blocking][ID {}] Async receiver dropped. Stopping blocking \
								 reader.",
								reader_task_terminal_id
							);

							break;
						}
					},

					Err(e) => {
						error!(
							"[Terminal PTY Reader Blocking][ID {}] Error reading from PTY master: {}. Stopping \
							 blocking reader.",
							reader_task_terminal_id, e
						);

						let _ = data_tx.blocking_send(Err(e));

						break;
					},
				}
			}
		});

		while let Some(read_result) = data_rx.recv().await {
			match read_result {
				Ok(bytes) if bytes.is_empty() => {
					// EOF signaled
					info!(
						"[Terminal PTY Reader][ID {}] Received EOF signal from blocking reader.",
						reader_task_terminal_id
					);

					break;
				},

				Ok(bytes) => {
					let data_str = String::from_utf8_lossy(&bytes);

					trace!(
						"[Terminal PTY Reader][ID {}] Read data (len {}): '{}...'",
						reader_task_terminal_id,
						bytes.len(),
						data_str.chars().take(70).collect::<String>()
					);

					// Send as [id, data_string]
					let payload = json!([reader_task_terminal_id, data_str.into_owned()]);

					if let Err(e) = vine::send_notification_to_sidecar(
						&reader_task_sidecar_id,
						"$acceptTerminalProcessData".to_string(),
						payload,
					)
					.await
					{
						error!(
							"[Terminal PTY Reader][ID {}] Failed to send $acceptTerminalProcessData to sidecar '{}': \
							 {}",
							reader_task_terminal_id, reader_task_sidecar_id, e
						);

						// If Vine pipe breaks, we might want to stop this
						// reader.
					}
				},

				Err(e) => {
					error!(
						"[Terminal PTY Reader][ID {}] Received error from blocking reader task: {}. Stopping.",
						reader_task_terminal_id, e
					);

					break;
				},
			}
		}

		info!(
			"[Terminal PTY Reader][ID {}] Async reader task finished.",
			reader_task_terminal_id
		);

		// Ensure blocking task finishes
		let _ = blocking_reader_thread_handle.await;
	});

	current_terminal_state.reader_task_handle = Some(Arc::new(TokioMutex::new(Some(reader_task_handle))));

	// 9. Spawn Process Wait Task (monitors PTY child process for exit).
	let wait_task_app_handle = app_handle.clone();

	// TODO: Make configurable
	let wait_task_sidecar_id = "cocoon-main".to_string();

	let wait_task_terminal_id = terminal_id;

	let process_wait_task_handle = tokio::spawn(async move {
		info!(
			"[Terminal Process Waiter][ID {}] Task started, awaiting PTY process exit.",
			wait_task_terminal_id
		);

		// `pty_child_process.wait()` is blocking. It must be run in `spawn_blocking`.
		let exit_status_result = tokio::task::spawn_blocking(move || pty_child_process.wait()).await;

		match exit_status_result {
			Ok(Ok(exit_status)) => {
				// Successfully got exit status from child process
				info!(
					"[Terminal Process Waiter][ID {}] PTY Process exited with status: {:?}",
					wait_task_terminal_id, exit_status
				);

				// VS Code `reason`: 0 for normal exit, 1 for error/signal exit.
				// `exit_code()`: Option<i32> on some platforms.
				let exit_code_val = exit_status.exit_code().map_or(Value::Null, |code| json!(code));

				let reason_code = if exit_status.success() { 0 } else { 1 };

				let payload = json!([wait_task_terminal_id, exit_code_val, reason_code]);

				if let Err(e) = vine::send_notification_to_sidecar(
					&wait_task_sidecar_id,
					"$acceptTerminalClosed".to_string(),
					payload,
				)
				.await
				{
					error!(
						"[Terminal Process Waiter][ID {}] Failed to send $acceptTerminalClosed (success exit) to \
						 sidecar '{}': {}",
						wait_task_terminal_id, wait_task_sidecar_id, e
					);
				}
			},

			Ok(Err(wait_err)) => {
				// Error from `pty_child_process.wait()` itself
				error!(
					"[Terminal Process Waiter][ID {}] Error from PTY process wait(): {}",
					wait_task_terminal_id, wait_err
				);

				// Reason 1 for error
				let payload = json!([wait_task_terminal_id, Value::Null, 1]);

				if let Err(e_ipc) = vine::send_notification_to_sidecar(
					&wait_task_sidecar_id,
					"$acceptTerminalClosed".to_string(),
					payload,
				)
				.await
				{
					error!(
						"[Terminal Process Waiter][ID {}] Failed to send $acceptTerminalClosed (wait error) to \
						 sidecar '{}': {}",
						wait_task_terminal_id, wait_task_sidecar_id, e_ipc
					);
				}
			},

			Err(join_err) => {
				// Error from `tokio::task::spawn_blocking().await` (e.g., task panicked)
				error!(
					"[Terminal Process Waiter][ID {}] JoinError waiting for PTY process exit task: {}. This is \
					 unexpected.",
					wait_task_terminal_id, join_err
				);

				// Send a generic error closure to Cocoon if the wait task itself failed.
				// Reason 1 for error
				let payload = json!([wait_task_terminal_id, Value::Null, 1]);

				if let Err(e_ipc) = vine::send_notification_to_sidecar(
					&wait_task_sidecar_id,
					"$acceptTerminalClosed".to_string(),
					payload,
				)
				.await
				{
					error!(
						"[Terminal Process Waiter][ID {}] Failed to send $acceptTerminalClosed (JoinError) to sidecar \
						 '{}': {}",
						wait_task_terminal_id, wait_task_sidecar_id, e_ipc
					);
				}
			},
		}

		// Cleanup terminal state from AppState after process has exited.
		let app_state_for_cleanup = wait_task_app_handle.state::<AppState>();

		if let Ok(mut active_terminals_guard) = app_state_for_cleanup.active_terminals.lock() {
			if active_terminals_guard.remove(&wait_task_terminal_id).is_some() {
				info!(
					"[Terminal Process Waiter][ID {}] Removed terminal from active list in AppState after exit.",
					wait_task_terminal_id
				);
			}
		} else {
			error!(
				"[Terminal Process Waiter][ID {}] Failed to lock active_terminals map for cleanup after exit (lock \
				 poisoned?).",
				wait_task_terminal_id
			);
		}

		info!(
			"[Terminal Process Waiter][ID {}] Process waiter task finished.",
			wait_task_terminal_id
		);
	});

	current_terminal_state.process_wait_handle = Some(Arc::new(TokioMutex::new(Some(process_wait_task_handle))));

	// 10. Store the fully initialized TerminalState in AppState.
	{
		let mut active_terminals_guard = app_state
			.active_terminals
			.lock()
			.map_err(|e| format_terminal_app_state_lock_error_for_rpc(e, "active_terminals map for final store"))?;

		active_terminals_guard.insert(terminal_id, Arc::new(StdMutex::new(current_terminal_state)));
	}

	// 11. Send initial notifications to Cocoon.
	// [id, name]
	let opened_payload = json!([terminal_id, terminal_name.clone()]);

	if let Err(e) = vine::send_notification_to_sidecar(
		// TODO: Make sidecar ID configurable
		"cocoon-main",
		"$acceptTerminalOpened".to_string(),
		opened_payload,
	)
	.await
	{
		error!(
			"[Terminal Handler Create] Failed to send $acceptTerminalOpened notification for term ID {}: {}",
			terminal_id, e
		);
	}

	// Re-fetch to get current_terminal_state's PID
	if let Some(pid_val) = app_state
		.active_terminals
		.lock()
		.map_err(|e| format_terminal_app_state_lock_error_for_rpc(e, "active_terminals map for PID read"))?
		.get(&terminal_id)
		.and_then(|term_arc| term_arc.lock().ok())
		.and_then(|term_guard| term_guard.os_process_id)
	{
		// [id, osProcessId]
		let pid_payload = json!([terminal_id, pid_val]);

		if let Err(e) =
			vine::send_notification_to_sidecar("cocoon-main", "$acceptTerminalProcessId".to_string(), pid_payload).await
		{
			error!(
				"[Terminal Handler Create] Failed to send $acceptTerminalProcessId notification for term ID {}: {}",
				terminal_id, e
			);
		}
	}

	// Return terminal ID, name, and OS PID to Cocoon.
	let final_os_pid_for_response = app_state
		.active_terminals
		.lock()
		.map_err(|e| format_terminal_app_state_lock_error_for_rpc(e, "active_terminals map for final PID response"))?
		.get(&terminal_id)
		.and_then(|term_arc| term_arc.lock().ok())
		.and_then(|term_guard| term_guard.os_process_id);

	Ok(json!({


		"id": terminal_id,

		"name": terminal_name,

		 // This will be `null` if PID was None
		"pid": final_os_pid_for_response
	}))
}

/// Handles the `$show` RPC call from Cocoon.
///
/// Emits a Tauri event (`mountain://terminal/reveal`) to request the frontend
/// (Sky) to make the specified terminal visible and optionally focus it.
///
/// # Arguments
/// * `app_handle` - The Tauri `AppHandle`.
/// * `args` - A `serde_json::Value` array: `[id: u64, preserveFocus?: boolean]`
///
/// # Returns
/// * `Ok(Value::Null)` on success.
/// * `Err(String)` if parameters are invalid or event emission fails.
pub async fn handle_show_terminal<R:Runtime>(app_handle:AppHandle<R>, args:Value) -> Result<Value, String> {
	let terminal_id = args.get(0).and_then(Value::as_u64).ok_or_else(|| {
		create_terminal_operation_rpc_error(
			"Missing or invalid terminal ID for $show operation".to_string(),
			Some("BADARG_ID"),
		)
	})?;

	let preserve_focus = args.get(1).and_then(Value::as_bool).unwrap_or(false);

	info!(
		"[Terminal Handler Show] RPC $show: id={}, preserveFocus={}",
		terminal_id, preserve_focus
	);

	// Emit an event for the Sky frontend to handle the UI aspect of showing the
	// terminal.
	app_handle
		.emit_all(
			// Custom event name for Sky
			"mountain://terminal/reveal",
			json!({"id": terminal_id, "preserveFocus": preserve_focus}),
		)
		.map_err(|e| {
			create_terminal_operation_rpc_error(
				format!("Failed to emit 'mountain://terminal/reveal' event: {}", e),
				Some("EMITFAIL"),
			)
		})?;

	Ok(Value::Null)
}

/// Handles the `$hide` RPC call from Cocoon.
///
/// This is often a no-op on the backend side, as terminal visibility is
/// primarily managed by the UI. It logs the request.
///
/// # Arguments
/// * `_app_handle` - The Tauri `AppHandle` (unused).
/// * `args` - A `serde_json::Value` array: `[id: u64]`
///
/// # Returns
/// * `Ok(Value::Null)`.
/// * `Err(String)` if parameters are invalid.
pub async fn handle_hide_terminal<R:Runtime>(
	// Unused for this handler
	_app_handle:AppHandle<R>,

	args:Value,
) -> Result<Value, String> {
	let terminal_id = args.get(0).and_then(Value::as_u64).ok_or_else(|| {
		create_terminal_operation_rpc_error(
			"Missing or invalid terminal ID for $hide operation".to_string(),
			Some("BADARG_ID"),
		)
	})?;

	info!(
		"[Terminal Handler Hide] RPC $hide: id={}. (This is typically a UI-only action).",
		terminal_id
	);

	// Hiding is usually managed by the frontend UI state.
	// Mountain backend doesn't need to do much other than acknowledge.
	// If there were backend state related to "active focus", it might be updated
	// here.
	Ok(Value::Null)
}

/// Handles the `$sendText` RPC call from Cocoon.
///
/// Sends the provided `text_to_send` to the PTY input of the specified
/// terminal via its dedicated MPSC channel and PTY writer task.
///
/// # Arguments
/// * `app_handle` - The Tauri `AppHandle`.
/// * `args` - A `serde_json::Value` array: `[id: u64, text: string]`
///
/// # Returns
/// * `Ok(Value::Null)` on successful queuing of text to the PTY writer task.
/// * `Err(String)` if parameters are invalid, channel not found, or send fails.
pub async fn handle_send_text_to_terminal<R:Runtime>(app_handle:AppHandle<R>, args:Value) -> Result<Value, String> {
	let terminal_id = args.get(0).and_then(Value::as_u64).ok_or_else(|| {
		create_terminal_operation_rpc_error(
			"Missing or invalid terminal ID for $sendText operation".to_string(),
			Some("BADARG_ID"),
		)
	})?;

	let text_to_send = args.get(1).and_then(Value::as_str).ok_or_else(|| {
		create_terminal_operation_rpc_error(
			"Missing or invalid text for $sendText operation".to_string(),
			Some("BADARG_TEXT"),
		)
	})?;

	info!(
		"[Terminal Handler SendText] RPC $sendText: id={}, text (first 30 chars)='{}...'",
		terminal_id,
		text_to_send.chars().take(30).collect::<String>()
	);

	let app_state = app_handle.state::<AppState>();

	let maybe_pty_input_sender_tx = {
		// Scope the lock for `active_terminals`
		let active_terminals_guard = app_state
			.active_terminals
			.lock()
			.map_err(|e| format_terminal_app_state_lock_error_for_rpc(e, "active_terminals map for sendText"))?;

		if let Some(terminal_state_arc) = active_terminals_guard.get(&terminal_id) {
			// Lock the individual TerminalState to get its pty_input_tx
			let terminal_state_guard = terminal_state_arc
				.lock()
				.map_err(|e| format_terminal_app_state_lock_error_for_rpc(e, "specific terminal state for sendText"))?;

			// Clone the Option<Sender>
			terminal_state_guard.pty_input_tx.clone()
		} else {
			// Terminal ID not found in active_terminals
			None
		}
	};

	if let Some(pty_input_tx) = maybe_pty_input_sender_tx {
		if let Err(e_send) = pty_input_tx.send(text_to_send.to_string()).await {
			let err_msg = format!(
				"Failed to send text to PTY writer task for terminal ID {}: channel closed or full. Error: {}",
				terminal_id, e_send
			);

			error!("[Terminal Handler SendText] {}", err_msg);

			return Err(create_terminal_operation_rpc_error(err_msg, Some("PIPEFAIL")));
		}

		trace!(
			"[Terminal Handler SendText] Text successfully sent to PTY writer task's MPSC channel for terminal ID: {}",
			terminal_id
		);
	} else {
		warn!(
			"[Terminal Handler SendText] No PTY input channel (pty_input_tx) found for terminal ID: {}. Terminal \
			 might be disposed or failed to initialize correctly.",
			terminal_id
		);

		return Err(create_terminal_operation_rpc_error(
			format!("Terminal with ID {} not found or is not ready for input.", terminal_id),
			Some("NOTFOUND_OR_NOTREADY"),
		));
	}

	Ok(Value::Null)
}

/// Handles the `$dispose` RPC call from Cocoon.
///
/// Terminates the PTY process associated with the given terminal ID and cleans
/// up its state and resources (tasks, channels).
///
/// # Arguments
/// * `app_handle` - The Tauri `AppHandle`.
/// * `args` - A `serde_json::Value` array: `[id: u64]`
///
/// # Returns
/// * `Ok(Value::Null)` on successful initiation of disposal.
/// * `Err(String)` if parameters are invalid or an internal error occurs.
pub async fn handle_dispose_terminal<R:Runtime>(app_handle:AppHandle<R>, args:Value) -> Result<Value, String> {
	let terminal_id = args.get(0).and_then(Value::as_u64).ok_or_else(|| {
		create_terminal_operation_rpc_error(
			"Missing or invalid terminal ID for $dispose operation".to_string(),
			Some("BADARG_ID"),
		)
	})?;

	info!(
		"[Terminal Handler Dispose] RPC $dispose received for terminal ID: {}",
		terminal_id
	);

	let app_state = app_handle.state::<AppState>();

	// Remove the terminal from the active map first to prevent new operations on
	// it.
	let terminal_arc_to_dispose_opt = {
		let mut active_terminals_guard = app_state
			.active_terminals
			.lock()
			.map_err(|e| format_terminal_app_state_lock_error_for_rpc(e, "active_terminals map in dispose"))?;

		// Returns Option<Arc<StdMutex<TerminalState>>>
		active_terminals_guard.remove(&terminal_id)
	};

	if let Some(terminal_state_arc) = terminal_arc_to_dispose_opt {
		info!(
			"[Terminal Handler Dispose] Initiating disposal for terminal ID: {}",
			terminal_id
		);

		// Now, operate on the removed TerminalState.
		let mut terminal_state_guard = terminal_state_arc
			.lock()
			.map_err(|e| format_terminal_app_state_lock_error_for_rpc(e, "specific terminal state in dispose"))?;

		// 1. Signal the PTY writer task to terminate by dropping its MPSC sender. The
		//    writer task will exit when its receiver channel detects the sender is
		//    dropped.
		if let Some(pty_input_tx) = terminal_state_guard.pty_input_tx.take() {
			// This closes the channel for the writer task.
			drop(pty_input_tx);

			debug!(
				"[Terminal Handler Dispose] PTY input sender dropped for terminal ID: {}",
				terminal_id
			);
		}

		// 2. Abort the PTY reader task.
		if let Some(reader_task_handle_arc) = terminal_state_guard.reader_task_handle.take() {
			// `reader_task_handle` is Arc<TokioMutex<Option<JoinHandle<()>>>>
			if let Some(join_handle) = reader_task_handle_arc.lock().await.take() {
				info!(
					"[Terminal Handler Dispose] Aborting PTY reader task for terminal ID: {}",
					terminal_id
				);

				// Request cancellation of the task.
				join_handle.abort();
			}
		}

		// 3. Abort the process wait task. This task is responsible for waiting on the
		//    PTY child process. Aborting it prevents it from sending a redundant
		//    `$acceptTerminalClosed` if we are already handling disposal. The PTY child
		//    process itself should be killed by its `PtyChild` handle being dropped (if
		//    `kill_on_drop` was set, or explicitly killed if `PtyChild` is available
		//    here). The `PtyChild` is captured in the process_wait_task. TODO: Ensure
		//    `PtyChild` is properly handled for killing the process. If `PtyChild` is
		//    not `kill_on_drop` by default from `portable-pty`, explicit kill might be
		//    needed here or by ensuring the `PtyChild` object is dropped. It is dropped
		//    when the process_wait_task exits or is aborted.
		if let Some(process_wait_task_handle_arc) = terminal_state_guard.process_wait_handle.take() {
			if let Some(join_handle) = process_wait_task_handle_arc.lock().await.take() {
				info!(
					"[Terminal Handler Dispose] Aborting PTY process waiter task for terminal ID: {}",
					terminal_id
				);

				join_handle.abort();
			}
		}

		// Note: The actual PTY child process (`PtyChild`) is owned by the
		// process_wait_task. When that task is aborted, the `PtyChild` will be
		// dropped. If `portable-pty`'s `PtyChild` implements `Drop` to kill the
		// process (or if `CommandBuilder::kill_on_drop(true)` was effective through
		// `PtyChild`), the OS process should terminate. Otherwise, an explicit kill
		// mechanism might be needed if `PtyChild` was stored in `TerminalState`.

		info!(
			"[Terminal Handler Dispose] Disposal tasks (abort reader/waiter, drop input sender) initiated for \
			 terminal ID: {}. Dependent tasks should now clean up.",
			terminal_id
		);
	} else {
		warn!(
			"[Terminal Handler Dispose] Dispose called for an unknown or already disposed terminal ID: {}. No action \
			 taken.",
			terminal_id
		);
	}

	Ok(Value::Null)
}

// --- Notification Handlers (From Cocoon Env Variable Collection Shim) ---
// These handlers are for notifications related to extensions wanting to modify
// the environment variables for future terminals. Currently, they are stubs.
// TODO: Implement full logic to store these environment changes in AppState
//       and apply them when new terminals are created via `$createTerminal`.

pub async fn handle_set_environment_variable_contribution<R:Runtime>(
	// Unused in stub
	_app_handle:AppHandle<R>,

	// Expected: { extensionId: string, variable: string, mutator: EnvVarMutatorDto }
	params:Value,
) -> Result<Value, String> {
	let extension_id = params.get("extensionId").and_then(Value::as_str).unwrap_or("unknown_extension");

	let variable_name = params.get("variable").and_then(Value::as_str).unwrap_or("unknown_variable");

	// `mutator` DTO: { type: number (Append=1, Prepend=2, Replace=3), value:
	// string, options?: IExtensionTerminalProfile }

	let mutator_details = params.get("mutator").cloned().unwrap_or(Value::Null);

	info!(
		"[Terminal EnvVar Handler] SetEnvContribution: Extension='{}', Variable='{}', MutatorDetails: {:?}",
		extension_id, variable_name, mutator_details
	);

	warn!(
		"[Terminal EnvVar Handler] STUB: Storing terminal environment variable contributions is not yet implemented. \
		 This change will not affect new terminals."
	);

	// TODO: Parse `mutator_details` and store the contribution in AppState, likely
	// in a map like: `AppState.terminal_env_contributions: HashMap<String /* extId
	// */, Vec<EnvVarChange>>` These contributions would then be applied by
	// `handle_create_terminal`.
	Ok(Value::Null)
}

pub async fn handle_delete_environment_variable_contribution<R:Runtime>(
	// Unused in stub
	_app_handle:AppHandle<R>,

	// Expected: { extensionId: string, variable: string }
	params:Value,
) -> Result<Value, String> {
	let extension_id = params.get("extensionId").and_then(Value::as_str).unwrap_or("unknown_extension");

	let variable_name = params.get("variable").and_then(Value::as_str).unwrap_or("unknown_variable");

	info!(
		"[Terminal EnvVar Handler] DeleteEnvContribution: Extension='{}', Variable='{}'",
		extension_id, variable_name
	);

	warn!(
		"[Terminal EnvVar Handler] STUB: Deleting terminal environment variable contributions is not yet implemented."
	);

	// TODO: Remove the specified variable contribution for the given extension from
	// AppState.
	Ok(Value::Null)
}

pub async fn handle_clear_environment_variable_collection_contributions<R:Runtime>(
	// Unused in stub
	_app_handle:AppHandle<R>,

	// Expected: { extensionId: string }
	params:Value,
) -> Result<Value, String> {
	let extension_id = params.get("extensionId").and_then(Value::as_str).unwrap_or("unknown_extension");

	info!(
		"[Terminal EnvVar Handler] ClearEnvContributionsForExtension: Extension='{}'",
		extension_id
	);

	warn!(
		"[Terminal EnvVar Handler] STUB: Clearing all terminal environment variable contributions for an extension is \
		 not yet implemented."
	);

	// TODO: Remove all environment variable contributions for the given extension
	// from AppState.
	Ok(Value::Null)
}
