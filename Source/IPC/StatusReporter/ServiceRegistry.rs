//! Service-discovery registry - the map of known services
//! and the schedule on which discovery refreshes it.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::IPC::StatusReporter::ServiceInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub services:HashMap<String, ServiceInfo::Struct>,

	pub last_discovery:u64,

	pub discovery_interval:u64,
}
