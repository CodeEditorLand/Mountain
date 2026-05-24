//! Aggregated DNS health snapshot (server / zone / forward
//! status + most recent error) returned by
//! `DnsGetHealthStatus`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub server_status:String,

	pub zone_status:String,

	pub forward_status:String,

	pub last_error:Option<String>,
}
