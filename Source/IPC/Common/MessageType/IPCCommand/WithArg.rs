//! `IPCCommand::WithArg`

use super::Struct;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::IPC::Common::MessageType::MessagePriority;

pub fn Fn(mut self, Arg:impl Into<String>) -> Struct {
		self.Args.push(Arg.into());

		self
	}
