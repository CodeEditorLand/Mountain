pub mod New;
pub mod WithArg;
pub mod WithParam;
pub mod WithPriority;

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
