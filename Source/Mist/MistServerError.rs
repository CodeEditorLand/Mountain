
// Defines the specific error types that can occur within the Mist WebSocket
// server.

#![allow(non_snake_case, non_camel_case_types)]

use thiserror::Error;
use tokio_tungstenite::tungstenite::Error as TungsteniteWsError;

#[derive(Debug, Error)]
pub enum MistServerError {
	#[error("WebSocket listener failed to bind or start: {0}")]
	ListenError(String),

	#[error("Failed to accept incoming TCP connection: {0}")]
	AcceptConnectionError(#[from] std::io::Error),

	#[error("WebSocket handshake error: {0}")]
	WebSocketHandshakeError(#[from] TungsteniteWsError),

	#[error("Failed to send message to client {ClientIdentifier}: {Details}")]
	MessageSendError { ClientIdentifier:u32, Details:String },

	#[error("Failed to receive message from client {ClientIdentifier}: {SourceError}")]
	MessageReceiveError { ClientIdentifier:u32, SourceError:TungsteniteWsError },

	#[error("WebSocket client connection {0} not found in active connections.")]
	ConnectionNotFound(u32),

	#[error("JSON processing error: {0}")]
	JsonProcessingError(#[from] serde_json::Error),

	#[error("Internal MPSC channel send error for client {ClientIdentifier}: {Details}")]
	InternalChannelSendError { ClientIdentifier:u32, Details:String },
}
