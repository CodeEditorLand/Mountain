//! `IPCMessage::WithPriority`

use super::Struct;
use serde::{Deserialize, Serialize};
use crate::IPC::Common::MessageType::MessagePriority;

pub fn Fn(mut self, Priority:MessagePriority::Enum) -> Struct {
		self.Priority = Priority;

		self
	}
