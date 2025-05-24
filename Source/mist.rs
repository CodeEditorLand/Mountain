// ---------------------------------------------------------------------------------------------
// Mountain Mist - Native WebSocket Server (mist.rs) [Feature Gated:
// mist_native]
// --------------------------------------------------------------------------------------------
// Implements an optional native WebSocket server within Mountain, allowing
// direct connections from clients (like the Sky frontend) if the `mist_native`
// feature flag is enabled during build. This serves as an alternative to
// requiring a separate Node.js-based Mist sidecar.
//
// Responsibilities:
// - Starting a `tokio::net::TcpListener` on a configured port.
// - Accepting incoming TCP connections.
// - Performing WebSocket handshakes using `tokio-tungstenite`.
// - Managing active client connections:
//   - Assigning unique connection IDs.
//   - Storing sender channels (`mpsc::Sender<WsMessage>`) for each connection
//     in a global map (`CONNECTIONS`).
// - Spawning tasks (`handle_connection`, reader/writer tasks) for each client
//   to:
//   - Read incoming WebSocket messages (`ws_receiver.next().await`).
//   - Process received messages (e.g., parse JSON, emit Tauri events like
//     `mist://message`).
//   - Send outgoing messages received via the MPSC channel
//     (`rx_from_mountain.recv().await`).
// - Providing a public function (`send_message_to_client`) for other Mountain
//   components to send string messages to specific clients via their connection
//   ID.
// - Handling connection cleanup (removing from `CONNECTIONS`, emitting
//   disconnect event) when a client disconnects or an error occurs.
//
// Key Interactions:
// - Started conditionally in `main.rs` via `tokio::spawn`.
// - Uses `tokio` (TCP, MPSC channels, tasks) and `tokio-tungstenite`.
// - Manages internal state (`CONNECTIONS`, `NEXT_CONN_ID`).
// - Emits Tauri events (`mist_client_connected`, `mist_client_disconnected`,

//   `mist://message`).
// - `send_message_to_client` can be called by handlers (e.g.,

//   `handlers::mist::handle_ws_send`) or effects to push data to the frontend.
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,
	net::SocketAddr,
	sync::{
		Arc,
		Mutex as StdMutex,
		atomic::{AtomicU32, Ordering},
	},
};

use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime};
use tokio::{
	net::{TcpListener, TcpStream},

	// Use tokio's MPSC channels for inter-task communication
	sync::mpsc,
};
use tokio_tungstenite::{
	WebSocketStream,
	accept_async,
	tungstenite::{Error as WsError, Message as WsMessage},
};

// Include track/vine if needed for error types or future interactions
use crate::{track, vine};

// --- Mist Error Type (Specific for this module) ---
#[derive(Debug, thiserror::Error)]
pub enum MistError {
	#[error("WebSocket listener failed: {0}")]
	ListenError(String),

	#[error("Failed to accept TCP connection: {0}")]
	AcceptError(std::io::Error),

	#[error("WebSocket handshake error: {0}")]
	HandshakeError(WsError),

	#[error("Failed to send message to client {0}: {1}")]
	SendError(u32, String),

	#[error("Failed to receive message from client {0}: {1}")]
	ReceiveError(u32, WsError),

	#[error("Client connection {0} not found")]
	ConnectionNotFound(u32),

	#[error("Serialization failed: {0}")]
	SerializationError(#[from] serde_json::Error),
}

// --- Server State ---

// Type alias for the map storing senders to client tasks.
// Key: Connection ID (u32), Value: Sender channel to the connection's writer
// task.
// Send WsMessage directly
type ConnectionMap = Arc<StdMutex<HashMap<u32, mpsc::Sender<WsMessage>>>>;

// Global map holding communication channels to active WebSocket clients.
static CONNECTIONS:Lazy<ConnectionMap> = Lazy::new(Default::default);

// Atomic counter for assigning unique connection IDs.
static NEXT_CONN_ID:Lazy<AtomicU32> = Lazy::new(|| AtomicU32::new(1));

// --- Server Initialization ---

/// Starts the native WebSocket server, listening for incoming connections.
/// This function should be called once during Mountain's setup.
pub async fn start_websocket_server<R:Runtime>(app_handle:AppHandle<R>) -> Result<(), MistError> {
	// TODO: Make port configurable via environment or AppState
	let port = 9001;

	let addr = format!("127.0.0.1:{}", port);

	// Bind the TCP listener.
	let listener = TcpListener::bind(&addr)
		.await
		.map_err(|e| MistError::ListenError(format!("Failed to bind to {}: {}", addr, e)))?;

	println!("[Mist] Native WebSocket server listening on {}", addr);

	// Accept connections in a loop.
	loop {
		match listener.accept().await {
			Ok((stream, peer_addr)) => {
				println!("[Mist] Accepted new TCP connection from: {}", peer_addr);

				let app_handle_clone = app_handle.clone();

				// Spawn a dedicated task to handle the WebSocket handshake and communication.
				tokio::spawn(handle_connection(stream, peer_addr, app_handle_clone));
			},

			Err(e) => {
				// Log error but continue accepting connections if possible.
				eprintln!("[Mist] Failed to accept TCP connection: {}", e);

				// Consider adding a small delay here if accept fails rapidly.
				// If the listener error is fatal, this loop might need to break
				// or return the error. For now, assume we can continue.
			},
		}
	}

	// Note: In this simple form, the server loop never exits unless the
	// listener fails fatally. Ok(())
}

// --- Connection Handling ---

/// Handles an individual WebSocket connection after the TCP connection is
/// accepted.
async fn handle_connection<R:Runtime>(stream:TcpStream, peer_addr:SocketAddr, app_handle:AppHandle<R>) {
	// Perform the WebSocket handshake.
	match accept_async(stream).await {
		Ok(ws_stream) => {
			// Assign a unique ID to this connection.
			let conn_id = NEXT_CONN_ID.fetch_add(1, Ordering::Relaxed);

			println!("[Mist] WebSocket handshake successful for {} [ID={}]", peer_addr, conn_id);

			// Split the WebSocket stream into a sender and receiver.
			let (mut ws_sender, mut ws_receiver) = ws_stream.split();

			// Create an MPSC channel for sending messages *to* this client *from* other
			// parts of Mountain. We send pre-formatted WsMessage objects for
			// flexibility.
			let (tx_to_client, mut rx_from_mountain) = mpsc::channel::<WsMessage>(100);

			// Register the sender channel in the global map.
			{
				// Scope the lock guard.
				let mut conns = CONNECTIONS.lock().unwrap();

				conns.insert(conn_id, tx_to_client);

				println!("[Mist] Registered sender for Conn ID {}", conn_id);
			}

			// Emit a Tauri event signalling a new client connection
			app_handle
				.emit_all(
					"mist_client_connected",
					json!({ "connId": conn_id, "peerAddr": peer_addr.to_string() }),
				)
				.ok();

			// --- Task: Reading messages FROM the client ---
			let app_handle_reader = app_handle.clone();

			let reader_task = tokio::spawn(async move {
				loop {
					match ws_receiver.next().await {
						Some(Ok(msg)) => {
							// Process different message types
							if msg.is_text() || msg.is_binary() {
								println!("[Mist Rx][Conn {}] Received message ({} bytes)", conn_id, msg.len());

								// --- Message Processing Logic ---
								// TODO: Parse the message payload (e.g., assume JSON text)
								let payload_value:Value = if msg.is_text() {
									serde_json::from_str(msg.to_text().unwrap_or_default()).unwrap_or_else(|e| {
										eprintln!(
											"[Mist Rx][Conn {}] Failed to parse text message as JSON: {}",
											conn_id, e
										);

										json!({ "parseError": e.to_string(), "original": msg.to_text().unwrap_or_default() })
									})
								} else {
									// Handle binary data if needed, maybe base64 encode for event?
									json!({ "binaryDataLength": msg.len() })
								};

								// --- Option A: Emit Tauri event ---
								// This is generally safer than directly calling Track from the network task.
								// Frontend or a dedicated Tauri-side listener can handle the event.
								if let Err(e) = app_handle_reader.emit_all(
									// Define a clear event name
									"mist://message",
									json!({"connId": conn_id, "payload": payload_value}),
								) {
									eprintln!("[Mist Rx][Conn {}] Failed to emit Tauri event: {}", conn_id, e);
								}

								// --- Option B: Send to a central queue/actor
								// (More complex) --- Not implemented
								// here.
							} else if msg.is_close() {
								println!("[Mist Rx][Conn {}] Received WebSocket close frame.", conn_id);

								// Exit loop on close frame
								break;
							} else if msg.is_ping() {
								println!("[Mist Rx][Conn {}] Received ping.", conn_id);

								// tokio-tungstenite handles sending pongs
								// automatically.
							} else if msg.is_pong() {
								println!("[Mist Rx][Conn {}] Received pong.", conn_id);
							}
						},

						Some(Err(WsError::ConnectionClosed)) => {
							println!("[Mist Rx][Conn {}] Connection closed normally by peer.", conn_id);

							// Exit loop
							break;
						},

						Some(Err(e)) => {
							eprintln!("[Mist Rx][Conn {}] Error receiving message: {}", conn_id, e);

							// Exit loop on error
							break;
						},

						None => {
							println!("[Mist Rx][Conn {}] WebSocket receiver stream ended.", conn_id);

							// Exit loop if stream ends
							break;
						},
					}
				}

				println!("[Mist Rx][Conn {}] Reader task finished.", conn_id);
			});

			// --- Task: Sending messages TO the client ---
			let writer_task = tokio::spawn(async move {
				while let Some(message_to_send) = rx_from_mountain.recv().await {
					if message_to_send.is_text() {
						// Log text messages carefully
						println!(
							"[Mist Tx][Conn {}] Sending message: {}",
							conn_id,
							message_to_send.to_text().unwrap_or("").chars().take(100).collect::<String>() /* Log truncated text */
						);
					} else {
						println!(
							"[Mist Tx][Conn {}] Sending non-text message (type: {:?}, len: {})",
							conn_id,
							message_to_send.type_id(),
							message_to_send.len()
						);
					}

					if let Err(e) = ws_sender.send(message_to_send).await {
						eprintln!("[Mist Tx][Conn {}] Error sending message: {}", conn_id, e);

						// If sending fails, the connection is likely broken. Stop the task.
						break;
					}
				}

				println!(
					"[Mist Tx][Conn {}] Writer task exiting (channel closed or send error).",
					conn_id
				);

				// Attempt graceful close on exit?
				// let _ = ws_sender.close().await;
			});

			// Wait for either the reader or writer task to finish.
			// This indicates the connection is closing or has errored.
			tokio::select! {

				_ = reader_task => { println!("[Mist][Conn {}] Reader task completed.", conn_id); },

				_ = writer_task => { println!("[Mist][Conn {}] Writer task completed.", conn_id); },

			}

			// --- Cleanup ---
			println!("[Mist] Cleaning up connection state for Conn ID {}", conn_id);

			{
				// Remove the sender channel from the global map.
				let mut conns = CONNECTIONS.lock().unwrap();

				if conns.remove(&conn_id).is_some() {
					println!("[Mist] Unregistered sender for Conn ID {}", conn_id);
				}
			}

			// Emit event signalling client disconnection
			app_handle
				.emit_all("mist_client_disconnected", json!({ "connId": conn_id }))
				.ok();

			// WebSocket stream (`ws_sender`, `ws_receiver`) is dropped here,

			// closing the connection.
		},

		Err(e) => {
			// Handshake failed.
			eprintln!("[Mist] WebSocket handshake error with {}: {}", peer_addr, e);
		},
	}
}

// --- Public API for Sending Messages ---

/// Sends a message (as a String) to a specific connected WebSocket client.
/// Called by other parts of Mountain (e.g., Track handlers).
pub async fn send_message_to_client(conn_id:u32, message:String) -> Result<(), MistError> {
	let sender = {
		let conns_guard = CONNECTIONS.lock().unwrap();

		// Clone the mpsc::Sender
		conns_guard.get(&conn_id).cloned()
	};

	if let Some(tx) = sender {
		// Wrap the string message into a tungstenite Text message.
		let ws_msg = WsMessage::Text(message);

		// Send it via the channel to the client's writer task.
		tx.send(ws_msg)
			.await
			.map_err(|e| MistError::SendError(conn_id, format!("MPSC channel send error: {}", e)))
	} else {
		Err(MistError::ConnectionNotFound(conn_id))
	}
}

// --- Example handler called by Track for ws_send command (Illustrative) ---
// This demonstrates how Track might use `send_message_to_client`.
pub async fn handle_ws_send<R:Runtime>(
	_app:AppHandle<R>,

	_window:Window<R>,

	// Assuming Track provides args as Vec<Value>
	args:Vec<Value>,
) -> Result<Value, String> {
	println!("[Mist Handler] Handling ws_send request: {:?}", args);

	// TODO: Robust arg parsing
	let conn_id_val = args.get(0).ok_or("Missing connection ID argument".to_string())?;

	let payload = args.get(1).cloned().ok_or("Missing payload argument".to_string())?;

	let conn_id = conn_id_val.as_u64().ok_or("Connection ID must be a number".to_string())? as u32;

	// Serialize the payload back to a JSON string to send
	let message_string = serde_json::to_string(&payload).map_err(|e| e.to_string())?;

	println!("[Mist Handler] Sending payload to Conn ID {}: {}", conn_id, message_string);

	send_message_to_client(conn_id, message_string)
        .await
         // Return null on success
		.map(|_| Value::Null)
         // Map MistError to String
		.map_err(|e| e.to_string())
}
