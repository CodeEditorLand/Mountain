// ---------------------------------------------------------------------------------------------
// Mountain Mist - Native WebSocket Server (mist.rs) [Feature Gated:
// mist_native]
// --------------------------------------------------------------------------------------------
// Implements an optional native WebSocket server within Mountain, allowing
// direct connections from clients (like the Sky frontend or other tools) if the
// `mist_native` feature flag is enabled during the build process. This serves
// as an alternative to requiring a separate Node.js-based "Mist" sidecar for
// WebSocket communication, potentially reducing complexity and dependencies.
//
// Responsibilities:
// - Starting a `tokio::net::TcpListener` on a configured network address and
//   port.
// - Asynchronously accepting incoming TCP connections.
// - Performing WebSocket handshakes with connecting clients using
//   `tokio-tungstenite`.
// - Managing active client connections:
//   - Assigning unique connection IDs (`conn_id`) to each client.
//   - Storing `tokio::sync::mpsc::Sender<WsMessage>` channels for each active
//     connection in a globally accessible, thread-safe map (`CONNECTIONS`).
//     This allows other parts of Mountain to send messages to specific clients.
// - Spawning dedicated asynchronous tasks (`handle_websocket_connection`, and
//   within it, reader/writer tasks) for each client to:
//   - Read incoming WebSocket messages (`ws_receiver.next().await`).
//   - Process received messages:
//     - Parse JSON text messages.
//     - Handle binary messages (currently logs length).
//     - Emit Tauri events (e.g., `mist://message`) containing the `conn_id` and
//       parsed payload, allowing other Mountain components or the Sky frontend
//       (if listening) to react.
//     - Respond to WebSocket PING frames (handled by `tokio-tungstenite`).
//   - Send outgoing messages:
//     - Messages are received via the client-specific MPSC channel
//       (`rx_from_mountain.recv().await`).
//     - These messages are then sent over the WebSocket connection to the
//       client.
// - Providing a public asynchronous function (`send_message_to_client_by_id`)
//   for other Mountain components (e.g., Tauri command handlers, effects) to
//   send string messages to specific WebSocket clients via their `conn_id`.
// - Handling connection cleanup:
//   - Removing the client's sender channel from the `CONNECTIONS` map upon
//     disconnection or error.
//   - Emitting Tauri events (`mist_client_connected`,

//     `mist_client_disconnected`) to signal connection lifecycle changes.
//
// Key Interactions:
// - Conditionally started in `main.rs` (within the Tauri `.setup()` hook) via
//   `tokio::spawn` if the `mist_native` feature is enabled.
// - Uses `tokio` for networking (TCP listener, streams), MPSC channels for
//   inter-task communication within a connection, and asynchronous tasks.
// - Uses `tokio-tungstenite` for WebSocket protocol handling (handshakes,

//   framing).
// - Manages internal global state: `CONNECTIONS` (HashMap of senders) and
//   `NEXT_CONN_ID` (atomic counter), using `once_cell::sync::Lazy` for lazy
//   initialization and `Arc<StdMutex<_>>` for thread-safe access to
//   `CONNECTIONS`.
// - Emits Tauri events using `AppHandle::emit` to decouple message processing
//   from direct calls into other Mountain systems.
// - The `send_message_to_client_by_id` function provides an outbound API for
//   other parts of Mountain.
// - An example handler `handle_ws_send_command` demonstrates how a Tauri
//   command could use `send_message_to_client_by_id`.
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,

	// For peer address
	net::SocketAddr,

	sync::{
		Arc,

		// Standard Mutex for the CONNECTIONS map
		Mutex as StdMutex,

		// For unique connection IDs
		atomic::{AtomicU32, Ordering as AtomicOrdering},
	},
};

// Traits for WebSocket send/receive
use futures_util::{SinkExt, StreamExt};
// For logging
use log::{debug, error, info, trace, warn};
// For lazy static initialization of global state
use once_cell::sync::Lazy;
// For constructing JSON payloads for Tauri events
use serde_json::{Value, json};
// Tauri essentials
use tauri::{AppHandle, Emitter, Runtime};
use tokio::{
	net::{TcpListener, TcpStream},

	// Tokio MPSC channels for inter-task communication
	sync::mpsc,
};
use tokio_tungstenite::{
	// For WebSocket server-side handshake
	accept_async,

	// WebSocket message types and errors
	tungstenite::{Error as TungsteniteWsError, Message as WsMessage},
};

// `track` and `vine` are not directly used in this module but were in the
// original context. If Mist needs to interact with them (e.g., for specific
// error types or IPC patterns), they could be re-introduced.
// use crate::{track, vine};

// --- Mist Error Type (Specific for WebSocket server operations) ---
#[derive(Debug, thiserror::Error)]
pub enum MistServerError {
	#[error("WebSocket listener failed to bind or start: {0}")]
	ListenError(String),

	#[error("Failed to accept incoming TCP connection: {0}")]
	// From TcpListener::accept
	AcceptTcpConnectionError(#[from] std::io::Error),

	#[error("WebSocket handshake error: {0}")]
	WebSocketHandshakeError(#[from] TungsteniteWsError),

	#[error("Failed to send message to client {client_id}: {details}")]
	MessageSendError { client_id:u32, details:String },

	#[error("Failed to receive message from client {client_id}: {source_error}")]
	MessageReceiveError { client_id:u32, source_error:TungsteniteWsError },

	#[error("WebSocket client connection {client_id} not found in active connections.")]
	ConnectionNotFound(u32),

	#[error("JSON serialization/deserialization error: {0}")]
	JsonProcessingError(#[from] serde_json::Error),

	#[error("Internal MPSC channel send error for client {client_id}: {details}")]
	InternalChannelSendError { client_id:u32, details:String },
}

// --- Server State (Global, Thread-Safe) ---

// Type alias for the map storing MPSC senders to individual client tasks.
// Key: Connection ID (u32).
// Value: `mpsc::Sender<WsMessage>` to send WebSocket messages to that client's
// writer task.
type ClientSenderChannelMap = Arc<StdMutex<HashMap<u32, mpsc::Sender<WsMessage>>>>;

// Global map holding communication channels (MPSC senders) to active WebSocket
// clients. `Lazy` ensures thread-safe, one-time initialization.
static ACTIVE_CLIENT_CONNECTIONS:Lazy<ClientSenderChannelMap> = Lazy::new(Default::default);

// Atomic counter for assigning unique IDs to new WebSocket connections.
static NEXT_CONNECTION_ID:Lazy<AtomicU32> = Lazy::new(|| AtomicU32::new(1));

// --- Server Initialization ---

/// Starts the native WebSocket server (Mist).
///
/// This function binds a TCP listener to a specified address and port, then
/// enters a loop to accept incoming TCP connections. Each accepted connection
/// is handled in a new asynchronous task.
///
/// # Argument
/// * `app_handle` - The Tauri `AppHandle`, passed to connection handlers for
///   emitting events.
///
/// # Returns
/// * `Ok(())` if the server starts listening successfully (though the function
///   loops indefinitely).
/// * `Err(MistServerError)` if the TCP listener fails to bind.
pub async fn start_websocket_server<R:Runtime>(app_handle:AppHandle<R>) -> Result<(), MistServerError> {
	// TODO: Make the listen address and port configurable (e.g., via AppState,

	// config file, or CLI args).
	let listen_port = 9001;

	let listen_addr = format!("127.0.0.1:{}", listen_port);

	// Bind the TCP listener to the address.
	let tcp_listener = TcpListener::bind(&listen_addr)
		.await
		.map_err(|e| MistServerError::ListenError(format!("Failed to bind to {}: {}", listen_addr, e)))?;

	info!(
		"[Mist Server] Native WebSocket server (Mist) started and listening on ws://{}",
		listen_addr
	);

	// Main server loop: continuously accept new TCP connections.
	loop {
		match tcp_listener.accept().await {
			Ok((tcp_stream, peer_addr)) => {
				info!("[Mist Server] Accepted new TCP connection from: {}", peer_addr);

				let app_handle_clone_for_connection = app_handle.clone();

				// Spawn a dedicated asynchronous task to handle this new connection.
				// This includes WebSocket handshake and subsequent message handling.
				tokio::spawn(handle_websocket_connection(
					tcp_stream,
					peer_addr,
					app_handle_clone_for_connection,
				));
			},

			Err(e) => {
				// Log the error but attempt to continue accepting connections.
				// If the listener itself fails catastrophically, this loop might exit,

				// or the `bind` call would have failed.
				error!(
					"[Mist Server] Failed to accept incoming TCP connection: {}. Server continues listening.",
					e
				);

				// TODO: Consider adding a small delay here if accept fails
				// rapidly due to transient issues       (e.g., resource
				// limits) to avoid a tight error loop.
				//       tokio::time::sleep(TokioDuration::from_millis(100)).
				// await;
			},
		}
	}

	// Note: In its current form, this function loops indefinitely. `Ok(())`
	// would only be reached if the loop somehow breaks, which it doesn't by
	// design here. A real server might have a shutdown signal to break the
	// loop.
}

// --- Connection Handling ---

/// Handles an individual WebSocket connection after a TCP connection has been
/// accepted.
///
/// This function performs the WebSocket handshake, sets up reader and writer
/// tasks for the connection, manages its lifecycle, and handles cleanup.
///
/// # Argument
/// * `tcp_stream` - The `TcpStream` for the accepted connection.
/// * `peer_addr` - The `SocketAddr` of the connected client.
/// * `app_handle` - The Tauri `AppHandle` for emitting events.
async fn handle_websocket_connection<R:Runtime>(tcp_stream:TcpStream, peer_addr:SocketAddr, app_handle:AppHandle<R>) {
	// Perform the WebSocket handshake.
	match accept_async(tcp_stream).await {
		Ok(websocket_stream) => {
			// Assign a unique ID to this successfully established WebSocket connection.
			let connection_id = NEXT_CONNECTION_ID.fetch_add(1, AtomicOrdering::Relaxed);

			info!(
				"[Mist Connection][ID {}] WebSocket handshake successful with client: {}",
				connection_id, peer_addr
			);

			// Split the WebSocket stream into a sender (Sink) and receiver (Stream).
			let (mut ws_message_sender, mut ws_message_receiver) = websocket_stream.split();

			// Create an MPSC channel for sending messages *to* this client *from* other
			// parts of Mountain. The sender part (`tx_to_client_writer_task`) will be
			// stored globally. The receiver part (`rx_from_mountain_for_writer_task`) is
			// used by this connection's writer task. Buffer size of 100 messages.
			let (tx_to_client_writer_task, mut rx_from_mountain_for_writer_task) = mpsc::channel::<WsMessage>(100);

			// Register the sender channel in the global map, keyed by connection_id.
			{
				// Scope for the Mutex lock guard.
				let mut connections_map_guard = ACTIVE_CLIENT_CONNECTIONS.lock().unwrap_or_else(|e| {
					// Handle poisoned lock, though this should be rare for global statics.
					error!(
						"[Mist Connection][ID {}] Global connections map lock poisoned! Attempting recovery. Error: {}",
						connection_id, e
					);

					e.into_inner()
				});

				connections_map_guard.insert(connection_id, tx_to_client_writer_task);

				debug!("[Mist Connection][ID {}] Registered MPSC sender in global map.", connection_id);
			}

			// Emit a Tauri event signalling a new client connection.
			if let Err(e) = app_handle.emit(
				// Event name for frontend/listeners
				"mist_client_connected",
				json!({ "connId": connection_id, "peerAddr": peer_addr.to_string() }),
			) {
				warn!(
					"[Mist Connection][ID {}] Failed to emit 'mist_client_connected' Tauri event: {}",
					connection_id, e
				);
			}

			// --- Task: Reading messages FROM the WebSocket client ---
			let app_handle_for_reader_task = app_handle.clone();

			let reader_task_join_handle = tokio::spawn(async move {
				info!("[Mist Reader Task][ID {}] Started for client: {}", connection_id, peer_addr);

				loop {
					match ws_message_receiver.next().await {
						Some(Ok(ws_msg)) => {
							// Successfully received a WebSocket message
							match ws_msg {
								WsMessage::Text(text_payload) => {
									trace!(
										"[Mist Reader Task][ID {}] Received Text message (len {}): '{}...'",
										connection_id,
										text_payload.len(),
										text_payload.chars().take(70).collect::<String>()
									);

									// Attempt to parse as JSON, as this is a common format for WebSocket APIs.
									let parsed_payload_value:Value = serde_json::from_str(&text_payload)
										.unwrap_or_else(|e_json| {
											warn!(
												"[Mist Reader Task][ID {}] Failed to parse received text message as \
												 JSON: {}. Original text: '{}...'",
												connection_id,
												e_json,
												text_payload.chars().take(100).collect::<String>()
											);

											// Fallback: treat as raw string if not JSON, or include error.
											json!({ "error": "JSONParseFailed", "details": e_json.to_string(), "originalText": text_payload })
										});

									// Emit a Tauri event with the received message.
									// Other parts of Mountain or Sky can listen for "mist://message".
									if let Err(e_emit) = app_handle_for_reader_task.emit(
										// Convention: "mist://<type>" or similar for events
										"mist://message",
										json!({"connId": connection_id, "payload": parsed_payload_value}),
									) {
										error!(
											"[Mist Reader Task][ID {}] Failed to emit 'mist://message' Tauri event: {}",
											connection_id, e_emit
										);
									}
								},

								WsMessage::Binary(bin_payload) => {
									trace!(
										"[Mist Reader Task][ID {}] Received Binary message (len {})",
										connection_id,
										bin_payload.len()
									);

									// TODO: Handle binary messages if the protocol requires it.
									//       For now, just acknowledge and potentially emit basic info.
									if let Err(e_emit) = app_handle_for_reader_task.emit(
										// Different event for binary?
										"mist://message_binary",
										json!({"connId": connection_id, "binaryDataLength": bin_payload.len()}),
									) {
										error!(
											"[Mist Reader Task][ID {}] Failed to emit 'mist://message_binary' Tauri \
											 event: {}",
											connection_id, e_emit
										);
									}
								},

								WsMessage::Ping(ping_payload) => {
									trace!(
										"[Mist Reader Task][ID {}] Received Ping frame. Tokio-tungstenite should \
										 auto-respond with Pong.",
										connection_id
									);

									// `tokio-tungstenite` typically handles
									// Pong responses automatically. If
									// manual Pong is needed:
									// ws_message_sender.
									// send(WsMessage::Pong(ping_payload)).
									// await;
								},

								WsMessage::Pong(_pong_payload) => {
									trace!("[Mist Reader Task][ID {}] Received Pong frame.", connection_id);

									// Usually for keep-alive checks initiated
									// by this server.
								},

								WsMessage::Close(close_frame_opt) => {
									info!(
										"[Mist Reader Task][ID {}] Received WebSocket Close frame from client: {:?}",
										connection_id, close_frame_opt
									);

									// Exit reader loop on Close frame.
									break;
								},

								WsMessage::Frame(_raw_frame) => {
									// This case is unlikely with typical `tokio-tungstenite` usage,

									// as it usually processes raw frames into typed WsMessages.
									trace!(
										"[Mist Reader Task][ID {}] Received raw Frame (should be processed by \
										 tungstenite).",
										connection_id
									);
								},
							}
						},

						Some(Err(TungsteniteWsError::ConnectionClosed)) => {
							info!(
								"[Mist Reader Task][ID {}] WebSocket connection closed by peer (Stream ended).",
								connection_id
							);

							// Normal closure by client.
							break;
						},

						Some(Err(ws_err)) => {
							error!(
								"[Mist Reader Task][ID {}] Error receiving message from WebSocket: {}. Terminating \
								 reader.",
								connection_id, ws_err
							);

							// Error condition, exit loop.
							break;
						},

						None => {
							// WebSocket stream ended without a Close frame.
							info!(
								"[Mist Reader Task][ID {}] WebSocket receiver stream ended (None received). Client \
								 may have disconnected abruptly.",
								connection_id
							);

							break;
						},
					}
				}

				info!(
					"[Mist Reader Task][ID {}] Reader task finished for client: {}",
					connection_id, peer_addr
				);
			});

			// --- Task: Sending messages TO the WebSocket client ---
			// This task listens on an MPSC channel for messages from other parts of
			// Mountain and sends them to the connected WebSocket client.
			let writer_task_join_handle = tokio::spawn(async move {
				info!("[Mist Writer Task][ID {}] Started for client: {}", connection_id, peer_addr);

				while let Some(message_to_send_to_client) = rx_from_mountain_for_writer_task.recv().await {
					if message_to_send_to_client.is_text() {
						trace!(
							"[Mist Writer Task][ID {}] Sending Text message: {}...",
							connection_id,
							message_to_send_to_client
								.to_text()
								.unwrap_or("")
								.chars()
								.take(70)
								.collect::<String>()
						);
					} else {
						trace!(
							"[Mist Writer Task][ID {}] Sending non-Text message (type: {:?}, len: {})",
							connection_id,
							message_to_send_to_client.type_id(),
							message_to_send_to_client.len()
						);
					}

					if let Err(e_send) = ws_message_sender.send(message_to_send_to_client).await {
						error!(
							"[Mist Writer Task][ID {}] Error sending message to WebSocket client: {}. Terminating \
							 writer.",
							connection_id, e_send
						);

						// If sending fails, the connection is likely broken. Stop this writer task.
						break;
					}
				}

				info!(
					"[Mist Writer Task][ID {}] Writer task exiting (MPSC channel closed or WebSocket send error) for \
					 client: {}",
					connection_id, peer_addr
				);

				// Attempt a graceful close of the WebSocket sender side if not
				// already closed. This might fail if the connection is
				// already broken. let _ = ws_message_sender.close().await;
			});

			// Wait for either the reader or writer task to finish.
			// Their completion signifies that the WebSocket connection is closing or has
			// errored.
			tokio::select! {


				_ = reader_task_join_handle => {


					info!("[Mist Connection][ID {}] Reader task completed. Connection closing.", connection_id);

				},


				_ = writer_task_join_handle => {


					info!("[Mist Connection][ID {}] Writer task completed. Connection closing.", connection_id);

				},


			}

			// --- Cleanup for this connection ---
			info!(
				"[Mist Connection][ID {}] Cleaning up connection state for client: {}",
				connection_id, peer_addr
			);

			{
				// Scope for Mutex lock guard.
				let mut connections_map_guard = ACTIVE_CLIENT_CONNECTIONS.lock().unwrap_or_else(|e| e.into_inner());

				if connections_map_guard.remove(&connection_id).is_some() {
					debug!(
						"[Mist Connection][ID {}] Unregistered MPSC sender from global map.",
						connection_id
					);
				}
			}

			// Emit a Tauri event signalling client disconnection.
			if let Err(e) = app_handle.emit(
				"mist_client_disconnected",
				json!({ "connId": connection_id, "peerAddr": peer_addr.to_string() }),
			) {
				warn!(
					"[Mist Connection][ID {}] Failed to emit 'mist_client_disconnected' Tauri event: {}",
					connection_id, e
				);
			}

			// The `websocket_stream` (and its split `ws_message_sender`,

			// `ws_message_receiver`) are dropped here when
			// `handle_websocket_connection` exits, which should trigger
			// the underlying TCP stream to close if not already closed.
		},

		Err(e_handshake) => {
			// WebSocket handshake failed.
			error!(
				"[Mist Connection] WebSocket handshake error with client {}: {}",
				peer_addr, e_handshake
			);
		},
	}
}

// --- Public API for Sending Messages from Mountain to Clients ---

/// Sends a string message to a specific connected WebSocket client, identified
/// by its `connection_id`.
///
/// This function is intended to be called by other parts of Mountain (e.g.,
///
///
/// Tauri command handlers, effects systems) that need to push data to a
/// specific client connected via the native Mist WebSocket server.
///
/// # Argument
/// * `connection_id` - The unique ID of the target WebSocket client connection.
/// * `message_string` - The string message to send. This will be wrapped in a
///   `WsMessage::Text`.
///
/// # Returns
/// * `Ok(())` if the message was successfully queued to the client's writer
///   task.
/// * `Err(MistServerError)` if the client connection is not found or the
///   internal MPSC channel send fails.
pub async fn send_message_to_client_by_id(connection_id:u32, message_string:String) -> Result<(), MistServerError> {
	let mpsc_sender_to_client_opt = {
		// Scope for lock
		let connections_map_guard = ACTIVE_CLIENT_CONNECTIONS.lock().unwrap_or_else(|e| e.into_inner());

		// Clone the mpsc::Sender
		connections_map_guard.get(&connection_id).cloned()
	};

	if let Some(mpsc_sender) = mpsc_sender_to_client_opt {
		// Wrap the string message into a tungstenite WsMessage::Text.
		let ws_text_message = WsMessage::Text(message_string);

		// Send it via the MPSC channel to the specific client's dedicated writer task.
		mpsc_sender.send(ws_text_message).await.map_err(|e_mpsc| {
			MistServerError::InternalChannelSendError {
				client_id:connection_id,

				details:format!("MPSC channel send failed for client writer task: {}", e_mpsc),
			}
		})
	} else {
		warn!(
			"[Mist Send API] Attempted to send message to non-existent or disconnected client ID: {}",
			connection_id
		);

		Err(MistServerError::ConnectionNotFound(connection_id))
	}
}

// --- Example Tauri Command Handler (Illustrative) ---
// This demonstrates how a Tauri command (e.g., invoked from Sky or another part
// of Mountain if it uses Tauri's invoke system) could use
// `send_message_to_client_by_id`.

/// **ILLUSTRATIVE:** Tauri command handler for sending a message via Mist.
///
/// This is an example of how other Mountain components might interact with
/// Mist.
///
/// # Argument
/// * `_app_handle` - The Tauri `AppHandle` (unused in this direct example).
/// * `_window` - The Tauri `Window` (unused).
/// * `args` - Expected to be a `Vec<Value>`: `[connection_id: u32, payload:
///   Value]`
///
/// # Returns
/// * `Ok(Value::Null)` on successful queuing of the message.
/// * `Err(String)` (JSON-RPC error string) on failure.
pub async fn handle_ws_send_command<R:Runtime>(
	// Unused
	_app_handle:AppHandle<R>,

	// Unused
	_window:Window<R>,

	// Expected: [conn_id_u32, payload_to_send_json_value]
	args:Vec<Value>,
) -> Result<Value, String> {
	debug!("[Mist Command Handler ws_send] Received request with args: {:?}", args);

	// --- Argument Parsing ---
	let connection_id_val = args
		.get(0)
		.ok_or_else(|| error_utils::rpc_param_error_string("mist_ws_send", "connection_id", "u32 number", Some(0)))?;

	let payload_to_send_json = args
		.get(1)
		.cloned()
		.ok_or_else(|| error_utils::rpc_param_error_string("mist_ws_send", "payload", "JSON value", Some(1)))?;

	let connection_id_u32 = connection_id_val.as_u64().ok_or_else(|| {
		error_utils::rpc_param_error_string("mist_ws_send", "connection_id", "u32 number (parsed as u64)", Some(0))
		// Cast u64 to u32
	})? as u32;

	// Serialize the payload JSON `Value` back into a JSON string to send over
	// WebSocket.
	let message_string_to_send = serde_json::to_string(&payload_to_send_json).map_err(|e_json| {
		error_utils::rpc_error_string(
			format!("Failed to serialize payload to JSON string for Mist: {}", e_json),
			Some("EJSON_SERIALIZE_MIST"),
		)
	})?;

	info!(
		"[Mist Command Handler ws_send] Sending payload to Mist Conn ID {}: {}...",
		connection_id_u32,
		message_string_to_send.chars().take(70).collect::<String>()
	);

	// Use the public API to send the message.
	send_message_to_client_by_id(connection_id_u32, message_string_to_send)
		.await
		 // Return `Ok(Value::Null)` on successful queuing
		.map(|_| Value::Null)
		.map_err(|mist_err| {


			// Map MistServerError to a JSON-RPC error string for the Tauri command response.
			error!(
				"[Mist Command Handler ws_send] Error sending message via Mist API: {}",


				mist_err
			);

			error_utils::rpc_error_string(
				format!("Mist WebSocket send error: {}", mist_err),


				 // Custom error code for this command failing
				Some("EMIST_SEND_CMD_FAIL"),


			)
		})
}
