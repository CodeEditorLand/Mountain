
//! IPC command request: command name + positional `Args` + named
//! `Params` map + a priority. Built through `new` and the `WithArg` /
//! `WithParam` / `WithPriority` builder shims.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::IPC::Common::MessageType::MessagePriority;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub Command:String,

	pub Args:Vec<String>,

	pub Params:HashMap<String, serde_json::Value>,

	pub Priority:MessagePriority::Enum,
}

impl Struct {
	pub fn new(Command:impl Into<String>) -> Self {
		Self {
			Command:Command.into(),

			Args:Vec::new(),

			Params:HashMap::new(),

			Priority:MessagePriority::Enum::Normal,
		}
	}

	pub fn WithArg(mut self, Arg:impl Into<String>) -> Self {
		self.Args.push(Arg.into());

		self
	}

	pub fn WithParam(mut self, Key:impl Into<String>, Value:serde_json::Value) -> Self {
		self.Params.insert(Key.into(), Value);

		self
	}

	pub fn WithPriority(mut self, Priority:MessagePriority::Enum) -> Self {
		self.Priority = Priority;

		self
	}
}
