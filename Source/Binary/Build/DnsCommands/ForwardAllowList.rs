//! Allowed external domains the DNS server may forward queries
//! to. Returned by `DnsGetForwardAllowlist`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub domains:Vec<String>,
}
