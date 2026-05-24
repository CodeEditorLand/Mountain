//! `SecurityEvent::New`

use super::Struct;
use serde::{Deserialize, Serialize};
use crate::IPC::Security::PermissionManager::SecurityEventType;

pub fn Fn(event_type:SecurityEventType::Enum, user_id:String, operation:String, details:Option<String>) -> Struct {
		Self { event_type, user_id, operation, timestamp:std::time::SystemTime::now(), details }
	}
