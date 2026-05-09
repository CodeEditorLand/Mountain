#![allow(non_snake_case)]

//! Audit log entry written by `Manager::Struct::log_security_event`.
//! Pairs a `SecurityEventType::Enum` with the user id,
//! attempted operation, timestamp, and free-form details.

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

impl Struct {

	pub fn new(event_type:SecurityEventType::Enum, user_id:String, operation:String, details:Option<String>) -> Self {

		Self { event_type, user_id, operation, timestamp:std::time::SystemTime::now(), details }
	}
}
