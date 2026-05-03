#![allow(non_snake_case)]

//! Basic DNS server information returned to the webview by
//! `dns_get_server_info`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsServerInfo {
	pub port:u16,
	pub is_running:bool,
	pub startup_time:String,
}
