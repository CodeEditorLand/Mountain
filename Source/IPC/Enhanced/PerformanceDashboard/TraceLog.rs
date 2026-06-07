//! Single in-span structured log: timestamp, message, level,
//! free-form fields. Carried inside `TraceSpan::Struct::logs`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::IPC::Enhanced::PerformanceDashboard::LogLevel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub timestamp:u64,

	pub message:String,

	pub level:LogLevel::Enum,

	pub fields:HashMap<String, String>,
}
