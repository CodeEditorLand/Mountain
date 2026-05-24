//! `IPCMessage::WithCorrelationId`

use super::Struct;
use serde::{Deserialize, Serialize};
use crate::IPC::Common::MessageType::MessagePriority;

pub fn Fn(mut self, CorrelationId:impl Into<String>) -> Struct {
		self.CorrelationId = Some(CorrelationId.into());

		self
	}
