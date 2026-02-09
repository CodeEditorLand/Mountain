//! # Message Type Definitions
//!
//! Provides core message structures for IPC communication.
//! Used for all IPC message passing between Wind and Mountain.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Standard IPC message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPCMessage {
	/// Unique message identifier
	pub id:String,
	/// Message type/command
	pub command:String,
	/// Message payload
	pub payload:serde_json::Value,
	/// Timestamp when message was created
	pub timestamp:u64,
	/// Optional correlation ID for request-response patterns
	pub correlation_id:Option<String>,
	/// Message priority
	pub priority:MessagePriority,
}

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
	/// Low priority, can be delayed
	Low = 0,
	/// Normal priority, standard processing
	Normal = 1,
	/// High priority, should be processed quickly
	High = 2,
	/// Critical priority, immediate processing required
	Critical = 3,
}

impl Default for MessagePriority {
	fn default() -> Self { Self::Normal }
}

/// IPC command request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPCCommand {
	/// Command identifier
	pub command:String,
	/// Command arguments
	pub args:Vec<String>,
	/// Command parameters as key-value pairs
	pub params:HashMap<String, serde_json::Value>,
	/// Message priority
	pub priority:MessagePriority,
}

impl IPCCommand {
	/// Create a new IPC command
	pub fn new(command:impl Into<String>) -> Self {
		Self {
			command:command.into(),
			args:Vec::new(),
			params:HashMap::new(),
			priority:MessagePriority::Normal,
		}
	}

	/// Add an argument to the command
	pub fn with_arg(mut self, arg:impl Into<String>) -> Self {
		self.args.push(arg.into());
		self
	}

	/// Add a parameter to the command
	pub fn with_param(mut self, key:impl Into<String>, value:serde_json::Value) -> Self {
		self.params.insert(key.into(), value);
		self
	}

	/// Set the message priority
	pub fn with_priority(mut self, priority:MessagePriority) -> Self {
		self.priority = priority;
		self
	}
}

/// IPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPCResponse {
	/// Correlation ID matching the original request
	pub correlation_id:String,
	/// Response data
	pub data:serde_json::Value,
	/// Whether the response indicates success
	pub success:bool,
	/// Error message if the request failed
	pub error:Option<String>,
	/// Response timestamp
	pub timestamp:u64,
}

impl IPCResponse {
	/// Create a successful response
	pub fn success(correlation_id:impl Into<String>, data:serde_json::Value) -> Self {
		Self {
			correlation_id:correlation_id.into(),
			data,
			success:true,
			error:None,
			timestamp:chrono::Utc::now().timestamp_millis() as u64,
		}
	}

	/// Create an error response
	pub fn error(correlation_id:impl Into<String>, error:impl Into<String>) -> Self {
		Self {
			correlation_id:correlation_id.into(),
			data:serde_json::Value::Null,
			success:false,
			error:Some(error.into()),
			timestamp:chrono::Utc::now().timestamp_millis() as u64,
		}
	}
}

impl IPCMessage {
	/// Create a new IPC message
	pub fn new(command:impl Into<String>) -> Self {
		Self {
			id:uuid::Uuid::new_v4().to_string(),
			command:command.into(),
			payload:serde_json::Value::Null,
			timestamp:chrono::Utc::now().timestamp_millis() as u64,
			correlation_id:None,
			priority:MessagePriority::Normal,
		}
	}

	/// Set the message payload
	pub fn with_payload(mut self, payload:serde_json::Value) -> Self {
		self.payload = payload;
		self
	}

	/// Set the correlation ID
	pub fn with_correlation_id(mut self, correlation_id:impl Into<String>) -> Self {
		self.correlation_id = Some(correlation_id.into());
		self
	}

	/// Set the message priority
	pub fn with_priority(mut self, priority:MessagePriority) -> Self {
		self.priority = priority;
		self
	}
}
