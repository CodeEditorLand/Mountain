//! `IPCCommand::WithPriority`

use super::Struct;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::IPC::Common::MessageType::MessagePriority;

pub fn Fn(mut self, Priority:MessagePriority::Enum) -> Struct {
		self.Priority = Priority;

		self
	}
