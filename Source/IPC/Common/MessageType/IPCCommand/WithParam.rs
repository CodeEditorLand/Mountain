//! `IPCCommand::WithParam`

use super::Struct;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::IPC::Common::MessageType::MessagePriority;

pub fn Fn(mut self, Key:impl Into<String>, Value:serde_json::Value) -> Struct {
		self.Params.insert(Key.into(), Value);

		self
	}
