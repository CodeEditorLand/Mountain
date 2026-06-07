//! Result envelope from a manual DNS resolution test
//! (`dns_resolve`). Carries the resolved address list and a
//! success flag so callers can branch without parsing strings.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsResolutionResult {

	pub domain:String,

	pub record_type:String,

	pub addresses:Vec<String>,

	pub ttl:u32,

	pub succeeded:bool,

	pub error:Option<String>,
}
