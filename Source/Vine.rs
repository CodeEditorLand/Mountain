// ---------------------------------------------------------------------------------------------
// Mountain Vine - Sidecar IPC Layer 
// --------------------------------------------------------------------------------------------
// Implements the Mountain-side of the custom Inter-Process Communication (IPC)
// protocol named "Vine". Vine is used for message-based communication between
// Mountain (the Tauri host application) and its sidecar processes (e.g., the
// Cocoon Node.js Extension Host). Communication occurs over the sidecar's
// standard input (stdin) and standard output (stdout) pipes, using framed JSON
// lines (each JSON message terminated by a newline).
//
// Responsibilities:
// - Setting up stdio pipes (`ChildStdin`, `ChildStdout`) for a newly spawned
//   sidecar process. This is initiated by `handlers::process_mgmt`.
// - Spawning asynchronous tasks (`spawn_vine_reader_task`,

//   `spawn_vine_writer_task`) for each sidecar connection. These tasks handle:
//   - Reading JSON lines from the sidecar's stdout.
//   - Writing JSON lines to the sidecar's stdin.
// - Defining the `VineMessage` structure and `VineMessageType` enum that define
//   the IPC message format (Request, Response, Error, Notification, Cancel).
// - Serializing outgoing `VineMessage`s to JSON strings and deserializing
//   incoming JSON strings into `VineMessage`s.
// - Managing request/response matching for calls initiated *from* Mountain *to*
//   sidecars. This involves:
//   - Generating unique request IDs (`NEXT_REQUEST_ID`).
//   - Storing `tokio::sync::oneshot::Sender` channels in a global map
//     (`PENDING_RESPONSES_FROM_SIDECARS`) keyed by request ID, to signal
//     completion or error of these requests.
//   - Handling timeouts for these requests.
// - Processing incoming messages from sidecars:
//   - `Request` messages: Routed to `track::dispatch_sidecar_request` for
//     further processing and execution within Mountain. The result is then sent
//     back to the sidecar as a `Response` or `Error` VineMessage.
//   - `Notification` messages: Processed based on their `method` field.
//     - Specific notifications like `"extHostReadyForInit"` or
//       `"extHostInitialized"` trigger global Tauri events (e.g.,

//       `vine://sidecar/ready`).
//     - Generic notifications are re-emitted as Tauri events like
//       `vine://notification/<sidecar_id>/<method>`.
//     - Log/error notifications from the sidecar are processed by Mountain's
//       logger.
//   - `Response`/`Error` messages: Matched against pending requests in
//     `PENDING_RESPONSES_FROM_SIDECARS` to resolve the corresponding
//     `oneshot::Sender`.
// - Providing public asynchronous functions for other Mountain components to
//   communicate with sidecars:
//   - `send_request_to_sidecar`: Sends a request and awaits a response or
//     error.
//   - `send_notification_to_sidecar`: Sends a fire-and-forget notification.
// - Managing the registration and unregistration of active sidecar
//   communication channels (primarily their writer MPSC senders in
//   `ACTIVE_SIDECAR_WRITERS`). This includes cleanup of pending requests if a
//   sidecar disconnects.
// - Monitoring the sidecar child process for exit and triggering cleanup.
//
// Key Interactions:
// - `setup_sidecar_communication` is called by `handlers::process_mgmt` when a
//   new sidecar is spawned.
// - Uses `tokio::process::Child` (and its `ChildStdin`, `ChildStdout`) for
//   interacting with the sidecar's stdio.
// - Uses `tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader}` for async
//   I/O.
// - Uses `serde_json` for serialization and deserialization of `VineMessage`s.
// - Manages internal global state maps (`ACTIVE_SIDECAR_WRITERS`,

//   `PENDING_RESPONSES_FROM_SIDECARS`) using `once_cell::sync::Lazy` and
//   `Arc<StdMutex<_>>` for thread-safe access.
// - Calls `track::dispatch_sidecar_request` to handle incoming RPC requests
//   from sidecars.
// - `send_request_to_sidecar` and `send_notification_to_sidecar` are called by
//   various Mountain components (e.g., `handlers::commands` for proxying,

//   `environment.rs` for IPC effects).
// - Emits Tauri events using `AppHandle::emit` to signal sidecar lifecycle
//   events or relay notifications.
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,

	sync::{
		Arc,

		// Standard Mutex for global state maps
		Mutex as StdMutex,

		MutexGuard,

		// For unique request IDs
		atomic::{AtomicU64, Ordering as AtomicOrdering},
	},

	// For request timeouts
	time::Duration,
};

// Logging facade
use log::{debug, error, info, trace, warn};
// For lazy static initialization
use once_cell::sync::Lazy;
// For message serialization/deserialization
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
// Tauri essentials
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tokio::{
	// Async I/O utilities
	io::{AsyncBufReadExt, AsyncWriteExt, BufReader},

	// For sidecar process stdio
	process::{Child, ChildStdin, ChildStdout},
	sync::{
		// Tokio MPSC channels for inter-task communication
		mpsc,
		// Tokio oneshot channels for request/response matching
		oneshot,
	},

	// For request timeouts
	time::timeout,
};

// For dispatching incoming requests from sidecars
use crate::runtime::AppRuntime;

// --- Vine Error Type ---
/// Defines errors specific to the Vine IPC layer.
#[derive(Debug, thiserror::Error)]
pub enum VineError {
	#[error("Sidecar process I/O error: {0}")]
	IoError(#[from] std::io::Error),

	#[error("Sidecar communication protocol error: {0}")]
	CommunicationProtocolError(String),

	#[error("Sidecar process '{sidecar_id}' not found, or its communication channel is closed.")]
	ProcessNotFoundOrChannelClosed { sidecar_id:String },

	#[error("JSON serialization error: {0}")]
	SerializationError(#[from] serde_json::Error),

	#[error("JSON deserialization error: {0}. Raw line (sample): '{1}'")]
	// Include raw line sample for debugging
	DeserializationError(String, String),

	#[error(
		"Request {request_id} to sidecar '{sidecar_id}' (method: '{method_name}') timed out after \
		 {timeout_duration_ms}ms"
	)]
	RequestTimeout { request_id:u64, sidecar_id:String, method_name:String, timeout_duration_ms:u64 },

	#[error("Internal state lock poisoned: {0}")]
	InternalLockError(String),

	#[error(
		"Request {request_id} to sidecar '{sidecar_id}' was cancelled because the sidecar's writer task failed or its \
		 communication channel was closed."
	)]
	RequestCancelledWriterTaskFailed { request_id:u64, sidecar_id:String },
}

// Implement From<PoisonError> for VineError to simplify lock error handling.
impl<T> From<std::sync::PoisonError<MutexGuard<'_, T>>> for VineError {
	fn from(e:std::sync::PoisonError<MutexGuard<'_, T>>) -> Self {
		VineError::InternalLockError(format!("Shared state lock poisoned: {}", e))
	}
}

// --- Vine Message Types and Structure ---

/// Defines the type of a Vine message.
/// Matches `RPCMessageType` in VS Code's `rpcProtocol.ts`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
// Ensures JSON keys match VS Code's convention
#[serde(rename_all = "camelCase")]
enum VineMessageType {
	// Client to Server
	Request = 1,
	// Server to Client (success)
	Response = 3,
	// Server to Client (failure)
	Error = 4,
	// Client to Server (to cancel a pending request)
	Cancel = 5,
	// Bidirectional, fire-and-forget
	Notification = 6,
}

/// Defines the structure of a Vine message exchanged over stdio.
/// Based on VS Code's `RPCMessage` structure.
#[derive(Serialize, Deserialize, Debug)]
struct VineMessage {
	// Type of the message
	msg_type:VineMessageType,

	#[serde(skip_serializing_if = "Option::is_none")]
	// Unique ID for Request/Response/Error, None for Notification/Cancel
	id: Option<u64>,

	#[serde(skip_serializing_if = "Option::is_none")]
	// Method name for Request/Notification
	method: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	// Parameters for Request/Notification, or result for Response
	params: Option<Value>,

	#[serde(skip_serializing_if = "Option::is_none")]
	// Error details for Error messages
	error: Option<Value>,
}

// --- Vine Internal State (Global, Thread-Safe) ---

/// Type alias for the map storing MPSC sender channels to sidecar writer tasks.
/// Key: `sidecar_id` (String).
/// Value: `mpsc::Sender<String>` (sends JSON strings to the writer task).
type SidecarWriterSenderMap = Arc<StdMutex<HashMap<String, mpsc::Sender<String>>>>;

/// Type alias for the map storing `oneshot::Sender` channels for pending
/// requests made *from* Mountain *to* sidecars.
/// Key: `request_id` (u64).
/// Value: `oneshot::Sender<Result<Value, VineError>>` (resolves the pending
/// request).
type PendingResponseCallbackMap = Arc<StdMutex<HashMap<u64, oneshot::Sender<Result<Value, VineError>>>>>;

// Global map holding MPSC senders to active sidecar writer tasks.
static ACTIVE_SIDECAR_WRITERS:Lazy<SidecarWriterSenderMap> = Lazy::new(Default::default);

// Global map holding `oneshot::Sender`s for pending responses from sidecars.
static PENDING_RESPONSES_FROM_SIDECARS:Lazy<PendingResponseCallbackMap> = Lazy::new(Default::default);

// Atomic counter for generating unique request IDs for messages sent from
// Mountain.
static NEXT_MOUNTAIN_REQUEST_ID:Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(1));

// --- Core Public Functions for Vine IPC Management ---

/// Sets up bi-directional IPC communication with a newly spawned sidecar
/// process.
///
/// This function takes ownership of the sidecar's `Child` process struct (or
/// at least its stdin/stdout handles), spawns dedicated reader and writer tasks
/// for JSON line-based communication, and registers the sidecar for future use.
///
/// # Argument
/// * `sidecar_id` - A unique string identifier for this sidecar instance.
/// * `sidecar_child_process` - The `tokio::process::Child` representing the
///   spawned sidecar. Its stdin and stdout will be taken.
/// * `app_handle` - The Tauri `AppHandle` for event emission and state access.
///
/// # Returns
/// * `Ok(())` if IPC setup is successfully initiated.
/// * `Err(VineError)` if stdin/stdout cannot be taken from the child process.
pub fn setup_sidecar_communication<R:Runtime>(
	sidecar_id:String,

	// Takes ownership to manage stdio and monitor exit
	mut sidecar_child_process:Child,

	app_handle:AppHandle<R>,
) -> Result<(), VineError> {
	let child_os_pid_str = sidecar_child_process
		.id()
		.map_or_else(|| "unknown_pid".to_string(), |pid| pid.to_string());

	info!(
		"[Vine Setup] Initializing IPC for sidecar '{}' [OS PID: {}]",
		sidecar_id, child_os_pid_str
	);

	// Take ownership of the child process's stdout and stdin pipes.
	let sidecar_stdout_pipe = sidecar_child_process.stdout.take().ok_or_else(|| {
		VineError::CommunicationProtocolError(format!("Failed to take stdout pipe for sidecar '{}'", sidecar_id))
	})?;

	let sidecar_stdin_pipe = sidecar_child_process.stdin.take().ok_or_else(|| {
		VineError::CommunicationProtocolError(format!("Failed to take stdin pipe for sidecar '{}'", sidecar_id))
	})?;

	// Create an MPSC channel for sending JSON strings to this sidecar's writer
	// task. Buffer size of 100 messages.
	let (mpsc_tx_to_sidecar_writer, mpsc_rx_for_sidecar_writer) = mpsc::channel::<String>(100);

	// Register the sender part of this channel globally.
	register_sidecar_writer_channel_sender(sidecar_id.clone(), mpsc_tx_to_sidecar_writer.clone());

	// Spawn the writer task.
	spawn_vine_writer_task(
		sidecar_id.clone(),
		child_os_pid_str.clone(),
		sidecar_stdin_pipe,
		mpsc_rx_for_sidecar_writer,
	);

	// Spawn the reader task.
	spawn_vine_reader_task(
		sidecar_id.clone(),
		child_os_pid_str.clone(),
		sidecar_stdout_pipe,
		// Pass Arc to global pending responses map
		PENDING_RESPONSES_FROM_SIDECARS.clone(),
		// Pass sender to allow reader to send responses directly back
		mpsc_tx_to_sidecar_writer,
		app_handle.clone(),
	);

	// Spawn a task to monitor the child process for exit.
	let sidecar_id_for_monitor = sidecar_id.clone();

	let child_os_pid_for_monitor = child_os_pid_str.clone();

	tokio::spawn(async move {
		match sidecar_child_process.wait().await {
			// `wait()` consumes `child_process`
			Ok(exit_status) => {
				info!(
					"[Vine ProcessMonitor] Sidecar '{}' [OS PID: {}] exited with status: {}",
					sidecar_id_for_monitor, child_os_pid_for_monitor, exit_status
				);
			},

			Err(e_wait) => {
				error!(
					"[Vine ProcessMonitor] Error waiting for sidecar '{}' [OS PID: {}] to exit: {}",
					sidecar_id_for_monitor, child_os_pid_for_monitor, e_wait
				);
			},
		}

		// Crucially, unregister the sidecar after it has exited to clean up resources
		// and cancel any pending requests to it.
		unregister_sidecar_communication_channel(&sidecar_id_for_monitor);
	});

	Ok(())
}

/// Registers the MPSC sender channel for a sidecar's writer task.
///
/// This allows other parts of Mountain to find and use this sender to queue
/// messages for that sidecar.
///
/// # Argument
/// * `sidecar_id` - The unique ID of the sidecar.
/// * `mpsc_sender_to_writer` - The `mpsc::Sender<String>` for the sidecar's
///   writer task.
fn register_sidecar_writer_channel_sender(sidecar_id:String, mpsc_sender_to_writer:mpsc::Sender<String>) {
	// Recover if poisoned
	let mut writers_map_guard = ACTIVE_SIDECAR_WRITERS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);

	info!(
		"[Vine Registry] Registering MPSC writer channel sender for sidecar: '{}'",
		sidecar_id
	);

	writers_map_guard.insert(sidecar_id, mpsc_sender_to_writer);
}

/// Unregisters a sidecar's communication channel and cleans up associated
/// resources.
///
/// This function should be called when a sidecar process exits or its
/// communication is intentionally terminated. It removes the sidecar's writer
/// MPSC sender from the global map and attempts to cancel any pending requests
/// that were targeting this sidecar.
///
/// # Argument
/// * `sidecar_id_to_unregister` - The ID of the sidecar to unregister.
pub fn unregister_sidecar_communication_channel(sidecar_id_to_unregister:&str) {
	let mut writers_map_guard = ACTIVE_SIDECAR_WRITERS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);

	if writers_map_guard.remove(sidecar_id_to_unregister).is_some() {
		info!(
			"[Vine Registry] Unregistered MPSC writer channel for sidecar: '{}'. Proceeding to clean up its pending \
			 requests...",
			sidecar_id_to_unregister
		);

		// Now, handle pending responses for this specific sidecar.
		// This requires PENDING_RESPONSES_FROM_SIDECARS to store which sidecar a
		// request targets, or a more complex cleanup if it doesn't.
		// For now, assuming a simplified cleanup that might affect other sidecars if
		// not careful. TODO: Refine PENDING_RESPONSES_FROM_SIDECARS to be keyed by
		// (sidecar_id, request_id)       or store sidecar_id with the oneshot::Sender
		// to selectively cancel.       Current broad cancellation is a
		// simplification.
		let mut pending_responses_map_guard = PENDING_RESPONSES_FROM_SIDECARS
			.lock()
			.unwrap_or_else(std::sync::PoisonError::into_inner);

		let mut requests_to_remove_ids:Vec<u64> = Vec::new();

		// Iterate and identify requests that were for the now-unregistered sidecar.
		// This is a placeholder loop; actual identification needs sidecar_id per
		// request.
		for (request_id, oneshot_tx) in pending_responses_map_guard.iter() {
			// **PLACEHOLDER CONDITION**: Assume all pending requests were for this sidecar.
			// In reality, you'd check `if request_target_sidecar_id ==
			// sidecar_id_to_unregister`.
			warn!(
				"[Vine Registry Cleanup] Cancelling pending request ID {} due to unregistration of sidecar '{}'. \
				 (Current cleanup logic might be too broad if multiple sidecars share pending map without \
				 distinction).",
				request_id, sidecar_id_to_unregister
			);

			// Attempt to send a cancellation error to the awaiter of this request.
			// Ignore error if the receiver (awaiting task) has already dropped.
			let _ = oneshot_tx.send(Err(VineError::RequestCancelledWriterTaskFailed {
				request_id:*request_id,

				sidecar_id:sidecar_id_to_unregister.to_string(),
			}));

			requests_to_remove_ids.push(*request_id);
		}

		let cancelled_count = requests_to_remove_ids.len();

		for req_id in requests_to_remove_ids {
			pending_responses_map_guard.remove(&req_id);
		}

		info!(
			"[Vine Registry Cleanup] Sidecar '{}' unregistered. {} pending requests were cancelled.",
			sidecar_id_to_unregister, cancelled_count
		);
	} else {
		debug!(
			"[Vine Registry] Unregister called for sidecar '{}', but no active writer channel found (already \
			 unregistered or never registered).",
			sidecar_id_to_unregister
		);
	}
}

/// Sends a request message to a specific sidecar and awaits its response or
/// error.
///
/// # Argument
/// * `target_sidecar_id` - The ID of the sidecar to send the request to.
/// * `method_name` - The RPC method name for the request.
/// * `params_val` - `serde_json::Value` containing parameters for the request.
/// * `timeout_duration_ms` - Timeout in milliseconds to wait for a response.
///
/// # Returns
/// * `Ok(Value)` with the response parameters if successful.
/// * `Err(VineError)` if the request fails, times out, or an IPC error occurs.
pub async fn send_request_to_sidecar(
	target_sidecar_id:&str,

	method_name:String,

	params_val:Value,

	timeout_duration_ms:u64,
) -> Result<Value, VineError> {
	let request_id = NEXT_MOUNTAIN_REQUEST_ID.fetch_add(1, AtomicOrdering::Relaxed);

	let vine_request_msg = VineMessage {
		msg_type:VineMessageType::Request,

		id:Some(request_id),

		// Clone for timeout error context
		method:Some(method_name.clone()),

		params:Some(params_val),

		error:None,
	};

	// Create a oneshot channel to receive the response for this request.
	let (response_tx_oneshot, response_rx_oneshot) = oneshot::channel();

	{
		// Scope for lock on PENDING_RESPONSES_FROM_SIDECARS
		// `?` uses From<PoisonError>
		let mut pending_map_guard = PENDING_RESPONSES_FROM_SIDECARS.lock()?;

		pending_map_guard.insert(request_id, response_tx_oneshot);
	}

	trace!(
		"[Vine SendRequest] ID={} Method='{}' ToSidecar='{}' ParamsSample='{}...'",
		request_id,
		// Safe, method is Some
		vine_request_msg.method.as_ref().unwrap(),
		target_sidecar_id,
		vine_request_msg
			.params
			.as_ref()
			.map_or("None", |p| &p.to_string().chars().take(50).collect::<String>())
	);

	// Send the raw message string to the sidecar's writer task.
	if let Err(e_send_raw) = send_raw_vine_message_to_sidecar_writer(target_sidecar_id, vine_request_msg).await {
		// If sending fails (e.g., writer task closed), remove the pending response
		// entry.
		PENDING_RESPONSES_FROM_SIDECARS.lock()?.remove(&request_id);

		error!(
			"[Vine SendRequest] Failed to send raw message for request ID={} to sidecar '{}': {}",
			request_id, target_sidecar_id, e_send_raw
		);

		return Err(e_send_raw);
	}

	// Await response on the oneshot channel with a timeout.
	match timeout(Duration::from_millis(timeout_duration_ms), response_rx_oneshot).await {
		Ok(Ok(response_result_from_sidecar)) => {
			// Successfully received a Result<Value, VineError> from the oneshot channel.
			// This is the Ok(Value) or Err(VineError::Communication)
			response_result_from_sidecar
		},

		Ok(Err(_oneshot_recv_err)) => {
			// `oneshot::Sender` was dropped without sending a value. This implies the
			// reader task for the sidecar exited or had an issue processing the response.
			warn!(
				"[Vine SendRequest] Response channel (oneshot) closed for request ID {} to sidecar '{}'. The \
				 sidecar's reader task may have exited or failed to resolve this request.",
				request_id, target_sidecar_id
			);

			// Cleanup
			PENDING_RESPONSES_FROM_SIDECARS.lock()?.remove(&request_id);

			Err(VineError::ProcessNotFoundOrChannelClosed { sidecar_id:target_sidecar_id.to_string() })
		},

		Err(_timeout_elapsed_err) => {
			// `tokio::time::error::Elapsed` means the timeout occurred.
			error!(
				"[Vine SendRequest] Request ID={} (Method='{}') to sidecar '{}' timed out after {}ms.",
				request_id, method_name, target_sidecar_id, timeout_duration_ms
			);

			// Cleanup
			PENDING_RESPONSES_FROM_SIDECARS.lock()?.remove(&request_id);

			Err(VineError::RequestTimeout {
				request_id,

				sidecar_id:target_sidecar_id.to_string(),

				// Already cloned
				method_name,

				timeout_duration_ms,
			})
		},
	}
}

/// Sends a fire-and-forget notification message to a specific sidecar.
///
/// # Argument
/// * `target_sidecar_id` - The ID of the sidecar to send the notification to.
/// * `method_name` - The RPC method name for the notification.
/// * `params_val` - `serde_json::Value` containing parameters for the
///   notification.
///
/// # Returns
/// * `Ok(())` if the notification was successfully queued to the sidecar's
///   writer task.
/// * `Err(VineError)` if an error occurs.
pub async fn send_notification_to_sidecar(
	target_sidecar_id:&str,

	method_name:String,

	params_val:Value,
) -> Result<(), VineError> {
	let vine_notification_msg = VineMessage {
		msg_type:VineMessageType::Notification,

		// Notifications don't have IDs
		id:None,

		method:Some(method_name.clone()),

		params:Some(params_val),

		error:None,
	};

	trace!(
		"[Vine SendNotification] Method='{}' ToSidecar='{}' ParamsSample='{}...'",
		method_name,
		target_sidecar_id,
		vine_notification_msg
			.params
			.as_ref()
			.map_or("None", |p| &p.to_string().chars().take(50).collect::<String>())
	);

	send_raw_vine_message_to_sidecar_writer(target_sidecar_id, vine_notification_msg).await
}

// --- Internal Helper Functions for Task Spawning and Message Processing ---

/// Spawns an asynchronous task that writes messages to a sidecar's stdin.
///
/// This task listens on an MPSC channel (`mpsc_rx_from_mountain`) for JSON
/// strings (serialized `VineMessage`s) and writes each one, followed by a
/// newline, to the sidecar's `ChildStdin`.
fn spawn_vine_writer_task(
	sidecar_id:String,

	// For logging
	child_os_pid_str:String,

	mut sidecar_stdin_pipe:ChildStdin,

	// Receives JSON strings to write
	mut mpsc_rx_from_mountain:mpsc::Receiver<String>,
) {
	tokio::spawn(async move {
		info!(
			"[Vine Writer Task][{}/PID {}] Writer task started.",
			sidecar_id, child_os_pid_str
		);

		while let Some(json_string_to_write) = mpsc_rx_from_mountain.recv().await {
			trace!(
				"[Vine Writer Task][{}] Writing to stdin: {}...",
				sidecar_id,
				json_string_to_write.chars().take(100).collect::<String>()
			);

			// Append newline as Vine uses JSON lines protocol.
			let line_to_write = json_string_to_write + "\n";

			if let Err(e_write) = sidecar_stdin_pipe.write_all(line_to_write.as_bytes()).await {
				error!(
					"[Vine Writer Task][{}/PID {}] Failed to write to sidecar stdin: {}. Stopping writer task.",
					sidecar_id, child_os_pid_str, e_write
				);

				// Exit loop on write error
				break;
			}

			if let Err(e_flush) = sidecar_stdin_pipe.flush().await {
				error!(
					"[Vine Writer Task][{}/PID {}] Failed to flush sidecar stdin: {}. Stopping writer task.",
					sidecar_id, child_os_pid_str, e_flush
				);

				// Exit loop on flush error
				break;
			}
		}

		info!(
			"[Vine Writer Task][{}/PID {}] MPSC channel closed or write error. Writer task exiting.",
			sidecar_id, child_os_pid_str
		);

		// When this task exits (e.g., MPSC channel closed), `sidecar_stdin_pipe` is
		// dropped, which should close the pipe to the child process.
		// Unregister sidecar if its writer task fails, as communication is broken.
		unregister_sidecar_communication_channel(&sidecar_id);
	});
}

/// Spawns an asynchronous task that reads messages from a sidecar's stdout.
///
/// This task reads newline-terminated JSON strings from the sidecar's
/// `ChildStdout`, deserializes them into `VineMessage`s, and processes them
/// accordingly (e.g., dispatching requests, resolving pending responses).
fn spawn_vine_reader_task<R:Runtime>(
	sidecar_id:String,

	// For logging
	child_os_pid_str:String,

	sidecar_stdout_pipe:ChildStdout,

	// Arc to global map
	pending_responses_map_arc:PendingResponseCallbackMap,

	// MPSC Sender to this sidecar's *own* writer task, used if reader needs to send a direct response (e.g., for an
	// incoming request).
	mpsc_tx_to_own_sidecar_writer:mpsc::Sender<String>,

	// For dispatching requests and emitting events
	app_handle:AppHandle<R>,
) {
	tokio::spawn(async move {
		info!(
			"[Vine Reader Task][{}/PID {}] Reader task started.",
			sidecar_id, child_os_pid_str
		);

		let mut stdout_buffered_reader = BufReader::new(sidecar_stdout_pipe);

		let mut line_buffer = String::new();

		loop {
			line_buffer.clear();

			match stdout_buffered_reader.read_line(&mut line_buffer).await {
				Ok(0) => {
					// 0 bytes read means EOF.
					info!(
						"[Vine Reader Task][{}/PID {}] Sidecar stdout stream ended (EOF). Sidecar process likely \
						 terminated.",
						sidecar_id, child_os_pid_str
					);

					// Exit loop
					break;
				},

				Ok(_) => {
					// Successfully read a line (or part of one if buffer too small, but read_line
					// handles this)
					trace!(
						"[Vine Reader Task][{}/PID {}] Raw line from stdout: {}...",
						sidecar_id,
						child_os_pid_str,
						line_buffer.trim_end().chars().take(100).collect::<String>()
					);

					// Process the received line (which should be a JSON string for a VineMessage).
					process_incoming_line_from_sidecar(
						&sidecar_id,
						// Pass borrowed line
						&line_buffer,
						pending_responses_map_arc.clone(),
						mpsc_tx_to_own_sidecar_writer.clone(),
						app_handle.clone(),
					)
					.await;
				},

				Err(e_read_line) => {
					error!(
						"[Vine Reader Task][{}/PID {}] Error reading line from sidecar stdout: {}. Stopping reader \
						 task.",
						sidecar_id, child_os_pid_str, e_read_line
					);

					// Exit loop on read error
					break;
				},
			}
		}

		info!(
			"[Vine Reader Task][{}/PID {}] Reader task exiting.",
			sidecar_id, child_os_pid_str
		);

		// When this task exits (e.g., stdout EOF or read error),

		// unregister the sidecar as communication is broken or finished.
		unregister_sidecar_communication_channel(&sidecar_id);
	});
}

/// Processes a single JSON line received from a sidecar.
///
/// Deserializes the line into a `VineMessage` and handles it based on its type.
async fn process_incoming_line_from_sidecar<R:Runtime>(
	sidecar_id_str:&str,

	json_line_str:&str,

	pending_responses_map_arc:PendingResponseCallbackMap,

	// For sending responses back to *this* sidecar
	mpsc_tx_to_own_sidecar_writer:mpsc::Sender<String>,

	app_handle:AppHandle<R>,
) {
	if json_line_str.trim().is_empty() {
		trace!("[Vine ProcessLine][{}] Ignoring empty line from sidecar.", sidecar_id_str);

		return;
	}

	match serde_json::from_str::<VineMessage>(json_line_str) {
		Ok(parsed_vine_message) => {
			trace!(
				"[Vine ProcessLine][{}] Parsed message: Type={:?}, ID={:?}, Method={:?}",
				sidecar_id_str, parsed_vine_message.msg_type, parsed_vine_message.id, parsed_vine_message.method
			);

			match parsed_vine_message.msg_type {
				VineMessageType::Response | VineMessageType::Error => {
					// This is a response or error for a request Mountain previously sent.
					if let Some(request_id) = parsed_vine_message.id {
						// Try to find and remove the pending oneshot sender for this request ID.
						let oneshot_callback_tx_opt = pending_responses_map_arc
							.lock()
							.unwrap_or_else(std::sync::PoisonError::into_inner)
							.remove(&request_id);

						if let Some(oneshot_callback_tx) = oneshot_callback_tx_opt {
							let result_for_awaiter = if parsed_vine_message.msg_type == VineMessageType::Response {
								debug!(
									"[Vine ProcessLine][{}] Received RESPONSE for Mountain request ID {}",
									sidecar_id_str, request_id
								);

								// Success result
								Ok(parsed_vine_message.params.unwrap_or(Value::Null))
							} else {
								// VineMessageType::Error
								warn!(
									"[Vine ProcessLine][{}] Received ERROR response from sidecar for Mountain request \
									 ID {}: {:?}",
									sidecar_id_str, request_id, parsed_vine_message.error
								);

								Err(VineError::CommunicationProtocolError(
									// Package sidecar's error
									parsed_vine_message
										.error
										.map(|e_val| e_val.to_string())
										.unwrap_or_else(|| "Unknown error from sidecar".to_string()),
								))
							};

							if oneshot_callback_tx.send(result_for_awaiter).is_err() {
								// This means the task awaiting the response (in send_request_to_sidecar)
								// has dropped its oneshot::Receiver, likely due to timeout or cancellation.
								warn!(
									"[Vine ProcessLine][{}] Failed to send processed response/error for request ID {} \
									 to internal awaiter (receiver dropped, request likely timed out or was \
									 cancelled).",
									sidecar_id_str, request_id
								);
							}
						} else {
							warn!(
								"[Vine ProcessLine][{}] Received response/error for unknown or already \
								 processed/timed-out Mountain request ID: {}",
								sidecar_id_str, request_id
							);
						}
					} else {
						error!(
							"[Vine ProcessLine][{}] Received Response/Error message from sidecar without a request \
							 ID: {:?}",
							sidecar_id_str, parsed_vine_message
						);
					}
				},

				VineMessageType::Request => {
					// This is a new request *from* the sidecar *to* Mountain.
					if let (Some(request_id_from_sidecar), Some(method_name), params_opt_val) = (
						parsed_vine_message.id,
						parsed_vine_message.method.clone(),
						parsed_vine_message.params,
					) {
						info!(
							"[Vine ProcessLine][{}] Received REQUEST from sidecar: ID={}, Method='{}'",
							sidecar_id_str, request_id_from_sidecar, method_name
						);

						let params_for_dispatch = params_opt_val.unwrap_or(Value::Null);

						// Construct payload for track::dispatch_sidecar_request
						let dispatch_request_payload = json!({ "method": method_name, "params": params_for_dispatch });

						// Get main window and AppRuntime for dispatching.
						// TODO: Consider if a "headless" mode without a main window is possible or if
						// sidecar requests always imply UI context.
						let main_window_opt = app_handle.get_webview_window("main");

						let app_runtime_state_opt = app_handle.try_state::<Arc<AppRuntime>>();

						let response_message_to_send_back:VineMessage = match (main_window_opt, app_runtime_state_opt) {

                            (Some(main_window), Some(app_runtime_state)) if app_runtime_state.inner().is_some() => {

								// Dispatch the request to Track.
                                match crate::track::dispatch_sidecar_request(
                                    app_handle.clone(),

                                    main_window,

                                    // Pass State<'_, Arc<AppRuntime>>
									app_runtime_state,

                                    sidecar_id_str.to_string(),

                                    dispatch_request_payload,

                                ).await {

                                    // Success from Track
									Ok(response_params_val) => VineMessage {

                                        msg_type: VineMessageType::Response,

                                        id: Some(request_id_from_sidecar),

                                        method: None, params: Some(response_params_val), error: None,

                                    },

                                    // Error string from Track
									Err(json_rpc_err_str_from_track) => {

                                        let error_payload_val = match serde_json::from_str::<Value>(&json_rpc_err_str_from_track) {

                                            // Already structured JSON error
											Ok(parsed_json_error) => parsed_json_error,

                                            // Fallback
											Err(_) => json!({ "message": json_rpc_err_str_from_track, "code": "EUNPARSABLE_TRACK_ERROR" }),

                                        };

                                        VineMessage {

                                            msg_type: VineMessageType::Error,

                                            id: Some(request_id_from_sidecar),

                                            method: None, params: None, error: Some(error_payload_val),

                                        }

                                    }

                                }

                            }

                            (None, _) => {

                                error!("[Vine ProcessLine][{}] Main window not found for processing request ID {} from sidecar.", sidecar_id_str, request_id_from_sidecar);

                                VineMessage { msg_type: VineMessageType::Error, id: Some(request_id_from_sidecar), method: None, params: None, error: Some(json!({"message": "Mountain internal error: Main window not found.", "code": "EINTERNAL_NOWINDOW"})) }

                            }

                            (_, None) | (_, Some(_)) /* if inner is None */ => {

                                error!("[Vine ProcessLine][{}] AppRuntime state unavailable/unusable for request ID {} from sidecar.", sidecar_id_str, request_id_from_sidecar);

                                VineMessage { msg_type: VineMessageType::Error, id: Some(request_id_from_sidecar), method: None, params: None, error: Some(json!({"message": "Mountain internal error: AppRuntime unavailable.", "code": "EINTERNAL_NORUNTIME"})) }

                            }

                        };

						// Send the response/error message back to the sidecar via its own writer task.
						if let Err(e_send_resp) = send_raw_vine_message_to_sidecar_writer_via_channel(
							&mpsc_tx_to_own_sidecar_writer,
							response_message_to_send_back,
						)
						.await
						{
							error!(
								"[Vine ProcessLine][{}] Failed to send response/error for sidecar request ID {} back \
								 to its writer task: {}",
								sidecar_id_str, request_id_from_sidecar, e_send_resp
							);
						}
					} else {
						error!(
							"[Vine ProcessLine][{}] Received invalid Request message from sidecar (missing ID or \
							 method): {:?}",
							sidecar_id_str, parsed_vine_message
						);
					}
				},

				VineMessageType::Notification => {
					if let Some(method_name_str) = parsed_vine_message.method {
						info!(
							"[Vine ProcessLine][{}] Received NOTIFICATION from sidecar: Method='{}'",
							sidecar_id_str, method_name_str
						);

						let params_val = parsed_vine_message.params.unwrap_or(Value::Null);

						// Handle specific, known notifications.
						match method_name_str.as_str() {
							"extHostReadyForInit" => {
								info!(
									"[Vine ProcessLine][{}] Sidecar signaled 'extHostReadyForInit'. Emitting Tauri \
									 event 'vine://sidecar/ready'.",
									sidecar_id_str
								);

								// Payload for this event is the sidecar_id string.
								if let Err(e_emit) = app_handle.emit("vine://sidecar/ready", sidecar_id_str.to_string())
								{
									error!(
										"[Vine ProcessLine][{}] Failed to emit 'vine://sidecar/ready' Tauri event: {}",
										sidecar_id_str, e_emit
									);
								}
							},

							"extHostInitialized" => {
								info!(
									"[Vine ProcessLine][{}] Sidecar signaled 'extHostInitialized'. Emitting Tauri \
									 event 'vine://sidecar/initialized'.",
									sidecar_id_str
								);

								if let Err(e_emit) =
									app_handle.emit("vine://sidecar/initialized", sidecar_id_str.to_string())
								{
									error!(
										"[Vine ProcessLine][{}] Failed to emit 'vine://sidecar/initialized' Tauri \
										 event: {}",
										sidecar_id_str, e_emit
									);
								}
							},

							"log" => {
								// General log messages from sidecar (standardized format)
								if let Some(log_entry_array) = params_val.as_array() {
									// VS Code LogLevel enum: Trace=0, Debug=1, Info=2, Warn=3, Error=4, Critical=5
									// Default to Info
									let level_num = log_entry_array.get(0).and_then(Value::as_u64).unwrap_or(2);

									let message_parts_vec:Vec<String> =
										log_entry_array.get(1..).map_or_else(Vec::new, |parts_slice| {
											parts_slice
												.iter()
												.map(|val| val.as_str().unwrap_or_else(|| &val.to_string()).to_string())
												.collect()
										});

									let log_message_str = message_parts_vec.join(" ");

									match level_num {
										0 => trace!("[SidecarLog::{}] {}", sidecar_id_str, log_message_str),

										1 => debug!("[SidecarLog::{}] {}", sidecar_id_str, log_message_str),

										2 => info!("[SidecarLog::{}] {}", sidecar_id_str, log_message_str),

										3 => warn!("[SidecarLog::{}] {}", sidecar_id_str, log_message_str),

										4 | 5 => error!("[SidecarLog::{}] {}", sidecar_id_str, log_message_str),

										_ => {
											info!(
												"[SidecarLog::{}] (UnknownLevel {}) {}",
												sidecar_id_str, level_num, log_message_str
											)
										},
									}
								} else {
									warn!(
										"[Vine ProcessLine][{}] Received malformed 'log' notification (params not an \
										 array): {:?}",
										sidecar_id_str, params_val
									);
								}
							},

							"error" | "extHostError" => {
								// General error reporting from sidecar
								error!("[SidecarError::{}] Reported error: {:?}", sidecar_id_str, params_val);

								if let Err(e_emit) = app_handle.emit(
									"vine://sidecar/error",
									json!({ "sidecarId": sidecar_id_str, "errorDetails": params_val }),
								) {
									error!(
										"[Vine ProcessLine][{}] Failed to emit 'vine://sidecar/error' Tauri event: {}",
										sidecar_id_str, e_emit
									);
								}
							},

							"rpcData" => {
								// For VSCode RPCProtocol, if Mountain needs to act as its client.
								if let Some(buffer_val_str) = params_val.get("buffer").and_then(Value::as_str) {
									trace!(
										"[Vine ProcessLine][{}] Received 'rpcData' notification from sidecar (buffer \
										 len: {}).",
										sidecar_id_str,
										buffer_val_str.len()
									);

									// TODO: If Mountain implements an RPC
									// client *to* the sidecar's RPC server
									// (less common),       this buffer
									// data would be fed into that client
									// instance.
								} else {
									warn!(
										"[Vine ProcessLine][{}] Received 'rpcData' notification without a 'buffer' \
										 string field: {:?}",
										sidecar_id_str, params_val
									);
								}
							},

							_ => {
								// Generic/unknown notification: re-emit as a namespaced Tauri event.
								let tauri_event_name =
									format!("vine://notification/{}/{}", sidecar_id_str, method_name_str);

								debug!(
									"[Vine ProcessLine][{}] Emitting generic Tauri event '{}' with payload (sample): \
									 {}...",
									sidecar_id_str,
									tauri_event_name,
									params_val.to_string().chars().take(50).collect::<String>()
								);

								if let Err(e_emit) = app_handle.emit(&tauri_event_name, params_val) {
									error!(
										"[Vine ProcessLine][{}] Failed to emit generic Tauri event '{}': {}",
										sidecar_id_str, tauri_event_name, e_emit
									);
								}
							},
						}
					} else {
						error!(
							"[Vine ProcessLine][{}] Received Notification message from sidecar without a method name: \
							 {:?}",
							sidecar_id_str, parsed_vine_message
						);
					}
				},

				VineMessageType::Cancel => {
					// Cancellation request *from* sidecar *to* Mountain
					if let Some(request_id_to_cancel) = parsed_vine_message.id {
						warn!(
							"[Vine ProcessLine][{}] Received CANCEL request from sidecar for Mountain's request ID: \
							 {}. Full cancellation propagation is not yet implemented.",
							sidecar_id_str, request_id_to_cancel
						);

						// TODO: Implement propagation of cancellation for
						// requests Mountain is handling for the sidecar.
						// This might involve:
						// 1. Finding the task associated with
						//    `request_id_to_cancel` (if Mountain tracks them).
						// 2. Signaling that task to abort or clean up (e.g.,

						//    via an AbortHandle or another channel).
					} else {
						warn!(
							"[Vine ProcessLine][{}] Received Cancel message from sidecar without a request ID: {:?}",
							sidecar_id_str, parsed_vine_message
						);
					}
				},
			}
		},

		Err(e_deserialize) => {
			// Include a sample of the raw line for easier debugging of deserialization
			// issues.
			let line_sample = json_line_str.trim_end().chars().take(200).collect::<String>();

			error!(
				"[Vine ProcessLine][{}] Failed to parse incoming line from sidecar as JSON VineMessage: '{}'. Raw \
				 line sample: '{}'",
				sidecar_id_str, e_deserialize, line_sample
			);

			// Store the error details for potential forwarding or logging.
			// For now, just logging it. Could emit a specific
			// "vine_protocol_error" event.
		},
	}
}

/// Sends a pre-serialized `VineMessage` (as a JSON string) to a sidecar's
/// writer task via its MPSC channel.
async fn send_raw_vine_message_to_sidecar_writer_via_channel(
	mpsc_tx_to_writer:&mpsc::Sender<String>,

	vine_message_to_send:VineMessage,
) -> Result<(), VineError> {
	let json_string_payload = serde_json::to_string(&vine_message_to_send).map_err(VineError::SerializationError)?;

	mpsc_tx_to_writer.send(json_string_payload).await.map_err(|e_mpsc_send| {
		VineError::CommunicationProtocolError(format!(
			"Failed to send message to sidecar's internal writer task MPSC channel: {}",
			e_mpsc_send
		))
	})
}

/// Looks up a sidecar's writer MPSC sender and sends a `VineMessage` to it.
async fn send_raw_vine_message_to_sidecar_writer(
	target_sidecar_id:&str,

	vine_message_to_send:VineMessage,
) -> Result<(), VineError> {
	let mpsc_sender_to_writer_opt = {
		// Scope for lock
		// `?` uses From<PoisonError>
		let writers_map_guard = ACTIVE_SIDECAR_WRITERS.lock()?;

		// Clone the mpsc::Sender
		writers_map_guard.get(target_sidecar_id).cloned()
	};

	if let Some(mpsc_tx) = mpsc_sender_to_writer_opt {
		let send_result = send_raw_vine_message_to_sidecar_writer_via_channel(&mpsc_tx, vine_message_to_send).await;

		if let Err(ref e_comm @ VineError::CommunicationProtocolError(_)) = send_result {
			// This specific error often means the writer task has exited (e.g., sidecar
			// stdin pipe broke).
			warn!(
				"[Vine SendRawToWriter] Send via MPSC channel failed for sidecar '{}'. Its writer task likely exited. \
				 Sidecar might be unresponsive or crashed. Error: {}",
				target_sidecar_id, e_comm
			);

			// No need to proactively clean PENDING_RESPONSES_FROM_SIDECARS
			// here, as timeouts or the reader task exiting (triggering
			// unregister_sidecar) should handle it.
		}

		send_result
	} else {
		error!(
			"[Vine SendRawToWriter] No active MPSC writer channel found for sidecar: '{}'. Sidecar might be \
			 unregistered, failed to start, or crashed.",
			target_sidecar_id
		);

		Err(VineError::ProcessNotFoundOrChannelClosed { sidecar_id:target_sidecar_id.to_string() })
	}
}
