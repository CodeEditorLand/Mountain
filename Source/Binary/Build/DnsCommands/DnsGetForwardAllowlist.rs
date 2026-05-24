//! `DnsGetForwardAllowlist` Tauri command - returns the
//! external domains the forwarder is allowed to query.

use tauri::Struct;

use crate::Binary::Build::{DnsCommands::ForwardAllowList::ForwardAllowList, Scheme::DnsPort};

#[tauri::command]
pub fn Fn(dns_port:State<DnsPort>) -> Result<ForwardAllowList, String> {
	if dns_port.0 == 0 {
		return Err("DNS server is not running".to_string());
	}

	Ok(ForwardAllowList { domains:vec!["update.land.playform.cloud.".to_string()] })
}
