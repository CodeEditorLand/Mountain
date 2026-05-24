pub mod New;
pub mod WithPayload;
pub mod WithCorrelationId;
pub mod WithPriority;

use serde::{Deserialize, Serialize};
use crate::IPC::Common::MessageType::MessagePriority;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub Id:String,

	pub Command:String,

	pub Payload:serde_json::Value,

	pub Timestamp:u64,

	pub CorrelationId:Option<String>,

	pub Priority:MessagePriority::Enum,
}
