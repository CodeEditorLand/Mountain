//! Basic DNS server information returned to the webview by
//! `DnsGetServerInfo`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub port:u16,

	pub is_running:bool,

	pub startup_time:String,
}
