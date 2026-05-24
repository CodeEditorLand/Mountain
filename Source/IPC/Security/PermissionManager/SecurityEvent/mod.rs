pub mod New;

use serde::{Deserialize, Serialize};
use crate::IPC::Security::PermissionManager::SecurityEventType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub event_type:SecurityEventType::Enum,

	pub user_id:String,

	pub operation:String,

	pub timestamp:std::time::SystemTime,

	pub details:Option<String>,
}
