//! `IPCMessage::New`

use super::Struct;
use serde::{Deserialize, Serialize};
use crate::IPC::Common::MessageType::MessagePriority;

pub fn Fn(Command:impl Into<String>) -> Struct {
		Self {
			Id:uuid::Uuid::new_v4().to_string(),

			Command:Command.into(),

			Payload:serde_json::Value::Null,

			Timestamp:chrono::Utc::now().timestamp_millis() as u64,

			CorrelationId:None,

			Priority:MessagePriority::Enum::Normal,
		}
	}
