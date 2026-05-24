//! `IPCCommand::New`

use super::Struct;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::IPC::Common::MessageType::MessagePriority;

pub fn Fn(Command:impl Into<String>) -> Struct {
		Self {
			Command:Command.into(),

			Args:Vec::new(),

			Params:HashMap::new(),

			Priority:MessagePriority::Enum::Normal,
		}
	}
