
//! Single-service descriptor produced by service discovery.
//! Carries identity, status, lifecycle timings, dependency
//! list, performance counters, and the gRPC endpoint.

use serde::{Deserialize, Serialize};

use crate::IPC::StatusReporter::{ServiceMetrics, ServiceStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub name:String,

	pub version:String,

	pub status:ServiceStatus::Enum,

	pub last_heartbeat:u64,

	pub uptime:u64,

	pub dependencies:Vec<String>,

	pub metrics:ServiceMetrics::Struct,

	pub endpoint:Option<String>,

	pub port:Option<u16>,
}
