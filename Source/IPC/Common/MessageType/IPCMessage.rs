
//! Standard IPC message: identifier, command name, JSON payload,
//! creation timestamp, optional correlation ID, and a priority. Built
//! through `new` and the `WithPayload` / `WithCorrelationId` /
//! `WithPriority` builder shims.

use serde::{Deserialize, Serialize};

use crate::IPC::Common::MessageType::MessagePriority;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub Id:String,

	pub Command:String,

	pub Payload:serde_json::Value,

	pub Timestamp:u64,

	pub CorrelationId:Option<String>,

	pub Priority:MessagePriority::Enum,
}

impl Struct {
	pub fn new(Command:impl Into<String>) -> Self {
		Self {
			Id:uuid::Uuid::new_v4().to_string(),

			Command:Command.into(),

			Payload:serde_json::Value::Null,

			Timestamp:chrono::Utc::now().timestamp_millis() as u64,

			CorrelationId:None,

			Priority:MessagePriority::Enum::Normal,
		}
	}

	pub fn WithPayload(mut self, Payload:serde_json::Value) -> Self {
		self.Payload = Payload;

		self
	}

	pub fn WithCorrelationId(mut self, CorrelationId:impl Into<String>) -> Self {
		self.CorrelationId = Some(CorrelationId.into());

		self
	}

	pub fn WithPriority(mut self, Priority:MessagePriority::Enum) -> Self {
		self.Priority = Priority;

		self
	}
}
