
//! Aggregated DNS health snapshot (server / zone / forward
//! status + most recent error) returned by
//! `dns_get_health_status`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsHealthStatus {
	pub server_status:String,

	pub zone_status:String,

	pub forward_status:String,

	pub last_error:Option<String>,
}
