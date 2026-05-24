//! `IPCMessage::WithPayload`

use super::Struct;
use serde::{Deserialize, Serialize};
use crate::IPC::Common::MessageType::MessagePriority;

pub fn Fn(mut self, Payload:serde_json::Value) -> Struct {
		self.Payload = Payload;

		self
	}
