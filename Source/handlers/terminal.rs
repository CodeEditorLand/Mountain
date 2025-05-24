// ---------------------------------------------------------------------------------------------
// Mountain Terminal Handlers (handlers/terminal.rs)
// --------------------------------------------------------------------------------------------
// Handles RPC requests and notifications related to terminals, proxied from
// Cocoon's terminal shims. Manages terminal processes or pseudo-terminals
// (PTYs) on the Mountain side.
//
// Responsibilities (MVP Focus):
// - Handling RPC calls related to creating/managing terminals ($createTerminal,
//   $show, $hide, $sendText, $dispose) - Currently STUBBED.
// - Handling notifications from the environment variable collection shim:
//   - `terminal_setEnvironmentVariable`
//   - `terminal_deleteEnvironmentVariable`
//   - `terminal_clearEnvironmentVariableCollection`
//   (These just log the received request for MVP).
// - Sending notifications back to Cocoon (`$accept...`) via Vine when terminal
//   state changes (STUBBED / TODO).
//
// Key Interactions:
// - Called by `track::dispatch_sidecar_request` for RPC methods and
//   notifications.
// - Would interact with `AppState` to store terminal instances/state (TODO).
// - Would use PTY libraries (e.g., `pty-process`) or `tokio::process::Command`
//   (TODO).
// - Would use `vine::send_notification` to send events back to Cocoon (TODO).
// --------------------------------------------------------------------------------------------

use log; // Use log crate for logging
use serde_json::{Value, json};
use tauri::{AppHandle, Runtime};
// TODO: Add imports for AppState, vine, PTY library, tokio::process etc. when
// implemented

// --- Placeholder State (Should live in AppState) ---
// Example structure for a terminal instance managed by Mountain
struct TerminalInstance {
	// id: u64, // Internal ID
	// name: String,
	// process_id: Option<u32>, // OS process ID
	// pty_handle: Option<Box<dyn PtyProcess + Send>>, // Example using pty-process trait
	// Or maybe child: Option<tokio::process::Child>, reader/writer handles etc.
}

// Map from internal terminal ID to the instance
// Use a thread-safe structure like Arc<Mutex<HashMap<u64, TerminalInstance>>>
// in AppState

// --- RPC Handlers (Called by Cocoon Shim) ---

/// Handles `$createTerminal` request.
/// Should spawn a PTY process based on options.
pub async fn handle_create_terminal<R:Runtime>(
	app:AppHandle<R>,
	options:Value, // Expects ICreateTerminalOptions structure
) -> Result<Value, String> {
	let name = options.get("name").and_then(|v| v.as_str()).unwrap_or("Terminal");
	log::info!("[Terminal Handler] RPC handle_create_terminal: name='{}'", name);
	// 1. TODO: Generate unique terminal ID (e.g., incrementing counter in AppState)
	let terminal_id = rand::random::<u64>(); // Placeholder ID generation

	// 2. TODO: Parse options (shellPath, shellArgs, cwd, env, etc.)

	// 3. TODO: Spawn actual PTY process using a library like `pty-process` or
	//    `portable-pty`. This involves setting up the command, environment
	//    (applying extension env vars), CWD, etc. let pty_system =
	//    native_pty_system(); let pair = pty_system.openpty(PtySize { rows: 24,
	//    cols: 80, ..Default::default() })?; let cmd = CommandBuilder::new("bash");
	//    // Example let mut child = pair.slave.spawn_command(cmd)?; let pty_reader
	//    = pair.master.try_clone_reader()?; let pty_writer =
	//    pair.master.try_clone_writer()?;

	// 4. TODO: Store TerminalInstance (including ID, name, process ID, PTY handles)
	//    in AppState map. let mut terminal_map =
	//    app.state::<AppState>().terminals.lock().unwrap();
	//    terminal_map.insert(terminal_id, TerminalInstance { ... });

	// 5. TODO: Spawn async task to read data from PTY (pty_reader) and send
	//    `$acceptTerminalProcessData` notifications via Vine to Cocoon.

	// 6. TODO: Spawn async task to monitor process exit (`child.wait()?`) and send
	//    `$acceptTerminalClosed` notification (including exit code/reason) via
	//    Vine. Also needs to clean up the entry in AppState.

	// 7. TODO: Send `$acceptTerminalOpened` and `$acceptTerminalProcessId`
	//    notifications via Vine.

	log::warn!("[Terminal Handler] STUB: Terminal process creation not implemented.");

	// Return initial info expected by Cocoon shim immediately
	Ok(json!({ "id": terminal_id, "name": name }))
}

/// Handles `$show` request.
/// Should signal the frontend UI to reveal the corresponding terminal.
pub async fn handle_show<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let id = args
		.get(0)
		.and_then(|v| v.as_u64())
		.ok_or_else(|| "Missing or invalid terminal ID (u64) for $show".to_string())?;
	let preserve_focus = args.get(1).and_then(|v| v.as_bool()).unwrap_or(false);
	log::info!(
		"[Terminal Handler] RPC handle_show: id={}, preserveFocus={}",
		id,
		preserve_focus
	);

	// 1. TODO: (Optional) Check if terminal with `id` exists in AppState.
	// 2. TODO: Trigger frontend UI to reveal the terminal panel/tab associated with
	//    ID. This typically involves emitting a Tauri event that the frontend
	//    listens for.
	app.emit_all("mountain://terminal/reveal", json!({"id": id, "preserveFocus": preserve_focus}))
		.map_err(|e| log::error!("Failed to emit terminal_reveal event: {}", e))
		.ok(); // Ignore emit errors for now

	log::warn!("[Terminal Handler] STUB: Frontend handling of terminal reveal event needed.");
	Ok(Value::Null)
}

/// Handles `$hide` request.
/// Should signal the frontend UI to hide the corresponding terminal (if
/// applicable).
pub async fn handle_hide<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let id = args
		.get(0)
		.and_then(|v| v.as_u64())
		.ok_or_else(|| "Missing or invalid terminal ID (u64) for $hide".to_string())?;
	log::info!("[Terminal Handler] RPC handle_hide: id={}", id);

	// 1. TODO: (Optional) Check if terminal with `id` exists in AppState.
	// 2. TODO: Trigger frontend UI to hide the terminal panel/tab (if needed).
	//    Often, hiding is managed by the user closing the panel directly.
	// app.emit_all("mountain://terminal/hide", json!({"id": id})).ok();

	log::warn!("[Terminal Handler] STUB: Hiding terminal UI via RPC might not be necessary.");
	Ok(Value::Null)
}

/// Handles `$sendText` request.
/// Should write the provided text to the terminal process's PTY input.
pub async fn handle_send_text<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let id = args
		.get(0)
		.and_then(|v| v.as_u64())
		.ok_or_else(|| "Missing or invalid terminal ID (u64) for $sendText".to_string())?;
	let text = args
		.get(1)
		.and_then(|v| v.as_str())
		.ok_or_else(|| "Missing or invalid text (string) for $sendText".to_string())?;

	log::info!(
		"[Terminal Handler] RPC handle_sendText: id={}, text='{}...'",
		id,
		text.chars().take(50).collect::<String>() // Log truncated text
	);

	// 1. TODO: Find terminal instance by ID in AppState (acquire lock).
	// 2. TODO: Get the PTY writer handle for the terminal.
	// 3. TODO: Write the `text` bytes to the PTY's stdin asynchronously. let mut
	//    writer = terminal_instance.pty_writer.lock().await; // Example lock
	//    writer.write_all(text.as_bytes()).await?;

	log::warn!("[Terminal Handler] STUB: Writing text to PTY not implemented.");
	Ok(Value::Null)
}

/// Handles `$dispose` request.
/// Should terminate the terminal process and clean up state.
pub async fn handle_dispose<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let id = args
		.get(0)
		.and_then(|v| v.as_u64())
		.ok_or_else(|| "Missing or invalid terminal ID (u64) for $dispose".to_string())?;
	log::info!("[Terminal Handler] RPC handle_dispose: id={}", id);

	// 1. TODO: Find terminal instance by ID in AppState (acquire lock).
	// 2. TODO: Kill the associated PTY process/task (e.g., `child.kill()`).
	// 3. TODO: Ensure any associated reader/monitor tasks are terminated.
	// 4. TODO: Remove the terminal instance from AppState map.
	// 5. TODO: Notify frontend UI to remove the terminal tab/panel if it hasn't
	//    already closed. (Often $acceptTerminalClosed handles this).

	log::warn!("[Terminal Handler] STUB: Disposing terminal process not implemented.");
	Ok(Value::Null)
}

// --- Notification Handlers (From Cocoon Env Variable Collection Shim via
// Vine/Track) --- These handlers are invoked when Cocoon calls
// sendNotificationToMountain for terminal env vars.

/// Handles `terminal_setEnvironmentVariable` notification.
/// Should store the variable mutation information per extension.
pub async fn handle_set_environment_variable<R:Runtime>(
	app:AppHandle<R>,
	params:Value, /* Expects { extensionId: string, variable: string, mutator: { value, type, scope?, precedence? },
	               * persistent: bool } */
) -> Result<Value, String> {
	// Extract parameters safely
	let extension_id = params.get("extensionId").and_then(Value::as_str).unwrap_or("unknown");
	let variable = params.get("variable").and_then(Value::as_str).unwrap_or("unknown");
	let mutator = params.get("mutator"); // This is the { value, type, ... } object
	let persistent = params.get("persistent").and_then(Value::as_bool).unwrap_or(true); // Get persistence flag

	log::info!(
		"[Terminal Env Handler] Received SetEnv Notification: Ext='{}', Var='{}', Persistent={}, Mutator={:?}",
		extension_id,
		variable,
		persistent,
		mutator // Log the whole mutator object for debugging
	);

	// --- TODO: Implement State Management ---
	// 1. Get access to AppState.
	// 2. Acquire lock on the environment variable storage map (e.g.,
	//    `extension_env_collections`). This map likely needs structure like:
	//    `HashMap<String(extensionId), HashMap<String(variableName),
	//    EnvVarMutator>>`
	// 3. Deserialize the `mutator` Value into a Rust struct (`EnvVarMutator`).
	// 4. Store the `EnvVarMutator` in the map under the correct extension ID and
	//    variable name.
	// 5. Handle persistence flag - if true, this change might need to be saved
	//    somewhere (e.g., separate JSON file).

	log::warn!("[Terminal Env Handler] STUB: Storing environment variable changes not implemented.");
	Ok(Value::Null) // Notifications don't typically require a specific response value
}

/// Handles `terminal_deleteEnvironmentVariable` notification.
/// Should remove the variable mutation information for the extension.
pub async fn handle_delete_environment_variable<R:Runtime>(
	app:AppHandle<R>,
	params:Value, // Expects { extensionId: string, variable: string, persistent: bool }
) -> Result<Value, String> {
	let extension_id = params.get("extensionId").and_then(Value::as_str).unwrap_or("unknown");
	let variable = params.get("variable").and_then(Value::as_str).unwrap_or("unknown");
	let persistent = params.get("persistent").and_then(Value::as_bool).unwrap_or(true);

	log::info!(
		"[Terminal Env Handler] Received DeleteEnv Notification: Ext='{}', Var='{}', Persistent={}",
		extension_id,
		variable,
		persistent
	);

	// --- TODO: Implement State Management ---
	// 1. Get access to AppState.
	// 2. Acquire lock on the environment variable storage map.
	// 3. Find the entry for `extension_id`.
	// 4. Remove the entry for `variable` from that extension's map.
	// 5. Handle persistence flag - update saved state if necessary.

	log::warn!("[Terminal Env Handler] STUB: Deleting environment variable changes not implemented.");
	Ok(Value::Null)
}

/// Handles `terminal_clearEnvironmentVariableCollection` notification.
/// Should clear all variable mutations for the extension.
pub async fn handle_clear_environment_variable_collection<R:Runtime>(
	app:AppHandle<R>,
	params:Value, // Expects { extensionId: string, persistent: bool }
) -> Result<Value, String> {
	let extension_id = params.get("extensionId").and_then(Value::as_str).unwrap_or("unknown");
	let persistent = params.get("persistent").and_then(Value::as_bool).unwrap_or(true);

	log::info!(
		"[Terminal Env Handler] Received ClearCollection Notification: Ext='{}', Persistent={}",
		extension_id,
		persistent
	);

	// --- TODO: Implement State Management ---
	// 1. Get access to AppState.
	// 2. Acquire lock on the environment variable storage map.
	// 3. Remove the entire inner map associated with `extension_id`.
	// 4. Handle persistence flag - update saved state if necessary.

	log::warn!("[Terminal Env Handler] STUB: Clearing environment variable collection not implemented.");
	Ok(Value::Null)
}

// NOTE: Getting env vars (`collection.get`) and handling the `onDidChange`
// event on the Cocoon side would require more complex state management and
// potentially event proxying from Mountain to Cocoon via Vine if changes
// could originate outside the specific extension's actions. These are omitted
// from the basic MVP shim/handlers for simplicity.
