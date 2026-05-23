//! Allowed external domains the DNS server may forward queries
//! to. Returned by `dns_get_forward_allowlist`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardAllowList {
	pub domains:Vec<String>,
}
