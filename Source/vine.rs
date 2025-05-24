// ---------------------------------------------------------------------------------------------
// Mountain Vine - Sidecar IPC Layer (vine.rs)
// --------------------------------------------------------------------------------------------
// Implements the Mountain-side of the custom IPC protocol ("Vine") used for
// communication between Mountain (Tauri host) and sidecar processes (e.g.,

// Cocoon Node.js Extension Host) over standard input/output (stdio) pipes. It
// provides reliable, asynchronous, message-based communication using framed
// JSON lines.
//
// Responsibilities:
// - Setting up stdio pipes (`ChildStdin`, `ChildStdout`) for a spawned sidecar
//   process.
// - Spawning async tasks (`spawn_vine_reader`, `spawn_vine_writer`) to handle
//   reading/writing JSON lines.
// - Defining the `VineMessage` structure and `VineMessageType` enum.
// - Serializing outgoing messages to JSON and deserializing incoming JSON
//   lines.
// - Managing request/response matching for calls *from* Mountain *to* sidecars
//   (`PENDING_RESPONSES`).
// - Routing incoming `Request` messages *from* sidecars to
//   `track::dispatch_sidecar_request`.
// - Processing incoming `Notification` messages *from* sidecars (readiness,
//   logs, errors) and emitting Tauri events or delegating.
// - Processing incoming `Response`/`Error` messages *from* sidecars to resolve
//   pending Mountain requests.
// - Providing public functions (`send_request_to_sidecar`,
//   `send_notification_to_sidecar`) for Mountain components.
// - Managing registration/unregistration of active sidecar communication
//   channels.
//
// Key Interactions:
// - Called by `handlers::process_mgmt` to set up communication for new
//   sidecars.
// - Uses `tokio::process` and `tokio::io` for async process IO.
// - Uses `serde_json` for serialization/deserialization.
// - Manages internal state maps (`SIDECAR_WRITERS`, `PENDING_RESPONSES`).
// - Calls `track::dispatch_sidecar_request` to handle incoming requests.
// - Called by various Mountain components via public functions.
// - Emits Tauri events (e.g., `vine://sidecar/ready`, `vine://sidecar/error`).
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,
	sync::{
		Arc,
		Mutex as StdMutex,
		MutexGuard,
		atomic::{AtomicU64, Ordering},
	},
	time::Duration,
};

use log::{debug, error, info, trace, warn};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime};
use tokio::{
	io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
	process::{Child, ChildStdin, ChildStdout},
	sync::{mpsc, oneshot},
	time::timeout,
};

use crate::{runtime::AppRuntime, track};

// --- Error Type ---
#[derive(Debug, thiserror::Error)]
pub enum VineError {
	#[error("Sidecar process I/O error: {0}")]
	Io(#[from] std::io::Error),

	#[error("Sidecar communication failed: {0}")]
	Communication(String),

	#[error("Sidecar process '{0}' not found or channel closed")]
	ProcessNotFoundOrClosed(String),

	#[error("Serialization error: {0}")]
	Serialization(#[from] serde_json::Error),

	#[error("Deserialization error: {0}")]
	Deserialization(String),

	#[error("Request {id} to sidecar '{sidecar_id}' (method: '{method}') timed out after {duration_ms}ms")]
	Timeout { id:u64, sidecar_id:String, method:String, duration_ms:u64 },

	#[error("Internal state lock error: {0}")]
	LockError(String),

	#[error("Request {id} cancelled because sidecar '{sidecar_id}' writer task failed or closed")]
	RequestCancelledWriterFailed { id:u64, sidecar_id:String },
}

impl<T> From<std::sync::PoisonError<MutexGuard<'_, T>>> for VineError {
	fn from(e:std::sync::PoisonError<MutexGuard<'_, T>>) -> Self {
		VineError::LockError(format!("State lock poisoned: {}", e))
	}
}

// --- Message Types ---
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum VineMessageType {
	Request = 1,

	Response = 3,

	Error = 4,

	Cancel = 5,

	Notification = 6,
}

// --- Message Structure ---
#[derive(Serialize, Deserialize, Debug)]
struct VineMessage {
	msg_type:VineMessageType,

	#[serde(skip_serializing_if = "Option::is_none")]
	id:Option<u64>,

	#[serde(skip_serializing_if = "Option::is_none")]
	method:Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	params:Option<Value>,

	#[serde(skip_serializing_if = "Option::is_none")]
	error:Option<Value>,
}

// --- Internal State ---
type SidecarWriterMap = Arc<StdMutex<HashMap<String, mpsc::Sender<String>>>>;

type PendingResponseMap = Arc<StdMutex<HashMap<u64, oneshot::Sender<Result<Value, VineError>>>>>;

static SIDECAR_WRITERS:Lazy<SidecarWriterMap> = Lazy::new(Default::default);

static PENDING_RESPONSES:Lazy<PendingResponseMap> = Lazy::new(Default::default);

static NEXT_REQUEST_ID:Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(1));

// --- Core Public Functions ---

pub fn setup_sidecar_communication<R:Runtime>(
	sidecar_id:String,

	mut child:Child,

	app_handle:AppHandle<R>,
) -> Result<(), VineError> {
	let child_pid_log = child.id().map_or_else(|| "unknown (child ID)".into(), |id| id.to_string());

	info!(
		"[Vine] Setting up communication for sidecar '{}' [PID: {}]",
		sidecar_id, child_pid_log
	);

	let stdout = child
		.stdout
		.take()
		.ok_or_else(|| VineError::Communication(format!("Failed to take stdout for sidecar '{}'", sidecar_id)))?;

	let stdin = child
		.stdin
		.take()
		.ok_or_else(|| VineError::Communication(format!("Failed to take stdin for sidecar '{}'", sidecar_id)))?;

	let (tx_to_sidecar_writer, rx_for_sidecar_writer) = mpsc::channel::<String>(100);

	register_sidecar_sender(sidecar_id.clone(), tx_to_sidecar_writer.clone());

	spawn_vine_writer(sidecar_id.clone(), child_pid_log.clone(), stdin, rx_for_sidecar_writer);

	spawn_vine_reader(
		sidecar_id.clone(),
		child_pid_log.clone(),
		stdout,
		PENDING_RESPONSES.clone(),
		tx_to_sidecar_writer,
		app_handle.clone(),
	);

	let sidecar_id_monitor = sidecar_id.clone();

	let child_pid_monitor = child_pid_log.clone();

	tokio::spawn(async move {
		match child.wait().await {
			Ok(status) => {
				info!(
					"[Vine Monitor] Sidecar '{}' [PID: {}] exited with status: {}",
					sidecar_id_monitor, child_pid_monitor, status
				)
			},

			Err(e) => {
				error!(
					"[Vine Monitor] Error waiting for sidecar '{}' [PID: {}] exit: {}",
					sidecar_id_monitor, child_pid_monitor, e
				)
			},
		}

		unregister_sidecar(&sidecar_id_monitor);
	});

	Ok(())
}

pub fn register_sidecar_sender(sidecar_id:String, tx:mpsc::Sender<String>) {
	let mut writers = SIDECAR_WRITERS.lock().unwrap_or_else(|e| e.into_inner());

	info!("[Vine] Registering writer for sidecar: {}", sidecar_id);

	writers.insert(sidecar_id, tx);
}

pub fn unregister_sidecar(sidecar_id:&str) {
	let mut writers_guard = SIDECAR_WRITERS.lock().unwrap_or_else(|e| e.into_inner());

	if writers_guard.remove(sidecar_id).is_some() {
		info!(
			"[Vine] Unregistered writer for sidecar: '{}'. Cleaning up its pending requests...",
			sidecar_id
		);

		let mut pending_guard = PENDING_RESPONSES.lock().unwrap_or_else(|e| e.into_inner());

		let mut requests_to_remove = Vec::new();

		// TODO: To make this more robust for multiple sidecars, PENDING_RESPONSES
		// would need to store (sidecar_id, oneshot_sender) or have separate maps.
		// For now, it assumes any pending request might be affected if *any* sidecar
		// unregisters, which is only safe if there's effectively one request-response
		// sidecar at a time or requests are globally unique.
		for (req_id, sender) in pending_guard.iter() {
			// This cancellation is broad. A more targeted approach would only cancel
			// requests *sent to* this specific sidecar_id. This requires
			// PENDING_RESPONSES to associate req_id with the target sidecar_id.
			warn!(
				"[Vine] Cancelling pending request ID {} because sidecar '{}' is being unregistered (broad \
				 cancellation).",
				req_id, sidecar_id
			);

			let _ = sender.send(Err(VineError::RequestCancelledWriterFailed {
				id:*req_id,

				sidecar_id:sidecar_id.to_string(),
			}));

			requests_to_remove.push(*req_id);
		}

		let cancelled_count = requests_to_remove.len();

		for req_id in requests_to_remove {
			pending_guard.remove(&req_id);
		}

		info!(
			"[Vine] Sidecar '{}' unregistered. {} pending requests potentially cancelled.",
			sidecar_id, cancelled_count
		);
	} else {
		debug!(
			"[Vine] Unregister called for sidecar '{}', but no writer found (already unregistered or never \
			 registered).",
			sidecar_id
		);
	}
}

pub async fn send_request_to_sidecar(
	sidecar_id:&str,

	method:String,

	params:Value,

	timeout_ms:u64,
) -> Result<Value, VineError> {
	let request_id = NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed);

	let request_msg = VineMessage {
		msg_type:VineMessageType::Request,

		id:Some(request_id),

		method:Some(method.clone()),

		params:Some(params),

		error:None,
	};

	let (response_tx, response_rx) = oneshot::channel();

	{
		let mut pending_guard = PENDING_RESPONSES.lock().map_err(VineError::from)?;

		pending_guard.insert(request_id, response_tx);
	}

	trace!(
		"[Vine SendRequest] ID={} Method='{}' To='{}' Params='{}...'",
		request_id,
		request_msg.method.as_ref().unwrap(),
		sidecar_id,
		request_msg
			.params
			.as_ref()
			.map_or("", |p| p.to_string().chars().take(50).collect::<String>())
	);

	if let Err(e) = send_raw_message_to_sidecar(sidecar_id, request_msg).await {
		PENDING_RESPONSES.lock().map_err(VineError::from)?.remove(&request_id);

		error!("[Vine SendRequest] Failed for ID={} To='{}': {}", request_id, sidecar_id, e);

		return Err(e);
	}

	match timeout(Duration::from_millis(timeout_ms), response_rx).await {
		Ok(Ok(res_result)) => res_result,

		Ok(Err(_recv_err)) => {
			warn!(
				"[Vine SendRequest] Response channel closed for ID={} (Sidecar '{}' likely exited/unregistered)",
				request_id, sidecar_id
			);

			PENDING_RESPONSES.lock().map_err(VineError::from)?.remove(&request_id);

			Err(VineError::ProcessNotFoundOrClosed(sidecar_id.to_string()))
		},

		Err(_timeout_err) => {
			error!(
				"[Vine SendRequest] ID={} Method='{}' To='{}' timed out after {}ms",
				request_id, method, sidecar_id, timeout_ms
			);

			PENDING_RESPONSES.lock().map_err(VineError::from)?.remove(&request_id);

			Err(VineError::Timeout { id:request_id, sidecar_id:sidecar_id.to_string(), method, duration_ms:timeout_ms })
		},
	}
}

pub async fn send_notification_to_sidecar(sidecar_id:&str, method:String, params:Value) -> Result<(), VineError> {
	let notification_msg = VineMessage {
		msg_type:VineMessageType::Notification,

		id:None,

		method:Some(method.clone()),

		params:Some(params),

		error:None,
	};

	trace!(
		"[Vine SendNotify] Method='{}' To='{}' Params='{}...'",
		method,
		sidecar_id,
		notification_msg
			.params
			.as_ref()
			.map_or("", |p| p.to_string().chars().take(50).collect::<String>())
	);

	send_raw_message_to_sidecar(sidecar_id, notification_msg).await
}

// --- Internal Helper Functions ---

fn spawn_vine_writer(
	sidecar_id:String,

	child_pid_log:String,

	mut stdin:ChildStdin,

	mut rx_from_mountain:mpsc::Receiver<String>,
) {
	tokio::spawn(async move {
		info!("[Vine Writer {}][PID: {}] Writer task started.", sidecar_id, child_pid_log);

		while let Some(json_string) = rx_from_mountain.recv().await {
			trace!(
				"[Vine Writer {}] Writing: {}...",
				sidecar_id,
				json_string.chars().take(100).collect::<String>()
			);

			if let Err(e) = stdin.write_all((json_string + "\n").as_bytes()).await {
				error!(
					"[Vine Writer {}][PID: {}] Failed to write to stdin: {}. Stopping writer.",
					sidecar_id, child_pid_log, e
				);

				break;
			}

			if let Err(e) = stdin.flush().await {
				error!(
					"[Vine Writer {}][PID: {}] Failed to flush stdin: {}. Stopping writer.",
					sidecar_id, child_pid_log, e
				);

				break;
			}
		}

		info!(
			"[Vine Writer {}][PID: {}] Channel closed or write error. Writer task exiting.",
			sidecar_id, child_pid_log
		);

		unregister_sidecar(&sidecar_id);

		drop(stdin);
	});
}

fn spawn_vine_reader<R:Runtime>(
	sidecar_id:String,

	child_pid_log:String,

	stdout:ChildStdout,

	pending_responses:PendingResponseMap,

	tx_to_own_sidecar_writer:mpsc::Sender<String>,

	app_handle:AppHandle<R>,
) {
	tokio::spawn(async move {
		info!("[Vine Reader {}][PID: {}] Reader task started.", sidecar_id, child_pid_log);

		let reader = BufReader::new(stdout);

		let mut lines = reader.lines();

		loop {
			match lines.next_line().await {
				Ok(Some(line)) => {
					trace!(
						"[Vine Reader {}][PID: {}] Raw line: {}...",
						sidecar_id,
						child_pid_log,
						line.chars().take(100).collect::<String>()
					);

					process_incoming_line_from_sidecar(
						&sidecar_id,
						&line,
						pending_responses.clone(),
						tx_to_own_sidecar_writer.clone(),
						app_handle.clone(),
					)
					.await;
				},

				Ok(None) => {
					info!(
						"[Vine Reader {}][PID: {}] Stdout stream ended (EOF).",
						sidecar_id, child_pid_log
					);

					break;
				},

				Err(e) => {
					error!(
						"[Vine Reader {}][PID: {}] Error reading stdout line: {}",
						sidecar_id, child_pid_log, e
					);

					break;
				},
			}
		}

		info!("[Vine Reader {}][PID: {}] Reader task exiting.", sidecar_id, child_pid_log);

		unregister_sidecar(&sidecar_id);
	});
}

async fn process_incoming_line_from_sidecar<R:Runtime>(
	sidecar_id:&str,

	line:&str,

	pending_responses:PendingResponseMap,

	tx_to_own_sidecar_writer:mpsc::Sender<String>,

	app_handle:AppHandle<R>,
) {
	if line.trim().is_empty() {
		trace!("[Vine ProcessLine {}] Ignoring empty line.", sidecar_id);

		return;
	}

	match serde_json::from_str::<VineMessage>(line) {
		Ok(message) => {
			trace!(
				"[Vine ProcessLine {}] Parsed message: Type={:?}, ID={:?}, Method={:?}",
				sidecar_id, message.msg_type, message.id, message.method
			);

			match message.msg_type {
				VineMessageType::Response | VineMessageType::Error => {
					if let Some(id) = message.id {
						let callback_tx = pending_responses.lock().unwrap_or_else(|e| e.into_inner()).remove(&id);

						if let Some(sender) = callback_tx {
							let result = if message.msg_type == VineMessageType::Response {
								debug!("[Vine ProcessLine {}] Received RESPONSE for req ID {}", sidecar_id, id);

								Ok(message.params.unwrap_or(Value::Null))
							} else {
								warn!(
									"[Vine ProcessLine {}] Received ERROR response for req ID {}: {:?}",
									sidecar_id, id, message.error
								);

								Err(VineError::Communication(
									message
										.error
										.map(|e_val| e_val.to_string())
										.unwrap_or_else(|| "Unknown error from sidecar".to_string()),
								))
							};

							if let Err(e) = sender.send(result) {
								error!(
									"[Vine ProcessLine {}] Failed to send processed response/error for req ID {} to \
									 internal awaiter: {:?}",
									sidecar_id, id, e
								);
							}
						} else {
							warn!(
								"[Vine ProcessLine {}] Received response/error for unknown or timed out request ID: {}",
								sidecar_id, id
							);
						}
					} else {
						error!(
							"[Vine ProcessLine {}] Received Response/Error message without an ID: {:?}",
							sidecar_id, message
						);
					}
				},

				VineMessageType::Request => {
					if let (Some(id), Some(method), params_opt) = (message.id, message.method.clone(), message.params) {
						info!(
							"[Vine ProcessLine {}] Received REQUEST ID={} Method='{}'",
							sidecar_id, id, method
						);

						let params = params_opt.unwrap_or(Value::Null);

						let request_payload = json!({ "method": method, "params": params });

						let response_message_to_sidecar:VineMessage = match app_handle.get_window("main") {
							Some(window) => {
								match app_handle.try_state::<Arc<AppRuntime>>() {
									// Use try_state
									Some(runtime_state) if runtime_state.inner().is_some() => {
										// Check Option inside Arc
										match track::dispatch_sidecar_request(
											app_handle.clone(),
											window,
											// Pass State<'_, Arc<AppRuntime>>
											runtime_state,
											sidecar_id.to_string(),
											request_payload,
										)
										.await
										{
											Ok(res_params) => {
												VineMessage {
													msg_type:VineMessageType::Response,

													id:Some(id),

													method:None,

													params:Some(res_params),

													error:None,
												}
											},

											Err(err_string) => {
												let error_payload = match serde_json::from_str::<Value>(&err_string) {
													Ok(json_error) => json_error,

													Err(_) => {
														json!({ "message": err_string, "code": "EUNKNOWN_HANDLER_ERROR" })
													},
												};

												VineMessage {
													msg_type:VineMessageType::Error,

													id:Some(id),

													method:None,

													params:None,

													error:Some(error_payload),
												}
											},
										}
									},

									_ => {
										// Catches None from try_state or if AppRuntime inside Arc is None
										error!(
											"[Vine ProcessLine {}] AppRuntime state unavailable or unusable for \
											 request ID {}.",
											sidecar_id, id
										);

										VineMessage {
											msg_type:VineMessageType::Error,

											id:Some(id),

											method:None,

											params:None,

											error:Some(
												json!({"message": "Internal Mountain Error: Runtime state unavailable/unusable", "code": "EINTERNAL_RUNTIME"}),
											),
										}
									},
								}
							},

							None => {
								error!(
									"[Vine ProcessLine {}] Main window not found for request ID {}.",
									sidecar_id, id
								);

								VineMessage {
									msg_type:VineMessageType::Error,

									id:Some(id),

									method:None,

									params:None,

									error:Some(
										json!({"message": "Main window not found in Mountain", "code": "ENOWINDOW"}),
									),
								}
							},
						};

						if let Err(e) = send_raw_message_to_sidecar_via_channel(
							&tx_to_own_sidecar_writer,
							response_message_to_sidecar,
						)
						.await
						{
							error!(
								"[Vine ProcessLine {}] Failed to send response for req ID {} to sidecar's writer \
								 task: {}",
								sidecar_id, id, e
							);
						}
					} else {
						error!(
							"[Vine ProcessLine {}] Received invalid Request message: {:?}",
							sidecar_id, message
						);
					}
				},

				VineMessageType::Notification => {
					if let Some(method) = message.method {
						info!("[Vine ProcessLine {}] Received NOTIFICATION Method='{}'", sidecar_id, method);

						let params = message.params.unwrap_or(Value::Null);

						match method.as_str() {
							"extHostReadyForInit" => {
								info!(
									"[Vine ProcessLine {}] Sidecar '{}' signaled 'extHostReadyForInit'.",
									sidecar_id, sidecar_id
								);

								if let Err(e) = app_handle.emit_all("vine://sidecar/ready", sidecar_id.to_string()) {
									error!(
										"[Vine ProcessLine {}] Failed to emit 'vine://sidecar/ready': {}",
										sidecar_id, e
									);
								}
							},

							"extHostInitialized" => {
								info!(
									"[Vine ProcessLine {}] Sidecar '{}' signaled 'extHostInitialized'.",
									sidecar_id, sidecar_id
								);

								if let Err(e) =
									app_handle.emit_all("vine://sidecar/initialized", sidecar_id.to_string())
								{
									error!(
										"[Vine ProcessLine {}] Failed to emit 'vine://sidecar/initialized': {}",
										sidecar_id, e
									);
								}
							},

							"log" => {
								if let Some(log_entry) = params.as_object() {
									let level = log_entry.get("level").and_then(Value::as_str).unwrap_or("INFO");

									let log_message = log_entry.get("message").and_then(Value::as_str).unwrap_or("");

									match level.to_uppercase().as_str() {
										"TRACE" => trace!("[{} Log] {}", sidecar_id, log_message),

										"DEBUG" => debug!("[{} Log] {}", sidecar_id, log_message),

										"INFO" => info!("[{} Log] {}", sidecar_id, log_message),

										"WARN" => warn!("[{} Log] {}", sidecar_id, log_message),

										"ERROR" => error!("[{} Log] {}", sidecar_id, log_message),

										_ => info!("[{} Log] (Unknown Level '{}') {}", sidecar_id, level, log_message),
									}
								} else {
									debug!("[{} Log] (Malformed) {:?}", sidecar_id, params);
								}
							},

							"error" | "extHostError" => {
								error!("[{} Error Reported] {:?}", sidecar_id, params);

								if let Err(e) = app_handle.emit_all(
									"vine://sidecar/error",
									json!({ "sidecarId": sidecar_id, "error": params }),
								) {
									error!(
										"[Vine ProcessLine {}] Failed to emit 'vine://sidecar/error': {}",
										sidecar_id, e
									);
								}
							},

							"rpcData" => {
								// VSCode RPCProtocol data for Mountain-as-client to Cocoon-as-server
								if let Some(_buffer_val_str) = params.get("buffer").and_then(Value::as_str) {
									trace!(
										"[Vine ProcessLine {}] Received rpcData notification (len: {})",
										sidecar_id,
										_buffer_val_str.len()
									);

									// TODO: If Mountain acts as an RPC client
									// to services in Cocoon, this data
									// needs to be fed into Mountain's RPC
									// client instance. This is not
									// part of the primary Mountain-as-server
									// flow.
								} else {
									warn!(
										"[Vine ProcessLine {}] Received rpcData without buffer string: {:?}",
										sidecar_id, params
									);
								}
							},

							_ => {
								let event_name = format!("vine://notification/{}/{}", sidecar_id, method);

								debug!(
									"[Vine ProcessLine {}] Emitting generic Tauri event: {} with payload (brief): \
									 {}...",
									sidecar_id,
									event_name,
									params.to_string().chars().take(50).collect::<String>()
								);

								if let Err(e) = app_handle.emit_all(&event_name, params) {
									error!(
										"[Vine ProcessLine {}] Failed to emit generic Tauri event '{}': {}",
										sidecar_id, event_name, e
									);
								}
							},
						}
					} else {
						error!(
							"[Vine ProcessLine {}] Received Notification message without method: {:?}",
							sidecar_id, message
						);
					}
				},

				VineMessageType::Cancel => {
					if let Some(id) = message.id {
						warn!(
							"[Vine ProcessLine {}] Received CANCEL request from sidecar for ID: {}. Cancellation not \
							 fully implemented.",
							sidecar_id, id
						);

						// TODO: Propagate cancellation to the task being
						// handled by `track::dispatch_sidecar_request`.
					} else {
						warn!(
							"[Vine ProcessLine {}] Received Cancel message without an ID: {:?}",
							sidecar_id, message
						);
					}
				},
			}
		},

		Err(e) => {
			error!(
				"[Vine ProcessLine {}] Failed to parse incoming line as JSON: '{}'. Raw line (first 200 chars): '{}'",
				sidecar_id,
				e,
				line.chars().take(200).collect::<String>()
			);
		},
	}
}

async fn send_raw_message_to_sidecar_via_channel(
	tx:&mpsc::Sender<String>,

	message:VineMessage,
) -> Result<(), VineError> {
	let json_string = serde_json::to_string(&message).map_err(VineError::Serialization)?;

	tx.send(json_string)
		.await
		.map_err(|e| VineError::Communication(format!("Failed to send message to sidecar writer task: {}", e)))
}

async fn send_raw_message_to_sidecar(sidecar_id:&str, message:VineMessage) -> Result<(), VineError> {
	let sender_opt = {
		let writers_guard = SIDECAR_WRITERS.lock().map_err(VineError::from)?;

		writers_guard.get(sidecar_id).cloned()
	};

	if let Some(tx) = sender_opt {
		let send_result = send_raw_message_to_sidecar_via_channel(&tx, message).await;

		if let Err(ref e @ VineError::Communication(_)) = send_result {
			warn!(
				"[Vine SendRaw] Send via channel failed for sidecar '{}', writer task likely exited: {}",
				sidecar_id, e
			);
		}

		send_result
	} else {
		error!(
			"[Vine SendRaw] No channel writer found for sidecar: '{}'. Sidecar might be unregistered or failed to \
			 start.",
			sidecar_id
		);

		Err(VineError::ProcessNotFoundOrClosed(sidecar_id.to_string()))
	}
}
