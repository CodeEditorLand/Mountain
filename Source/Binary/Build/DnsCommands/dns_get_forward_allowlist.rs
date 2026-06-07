//! `dns_get_forward_allowlist` Tauri command - returns the
//! external domains the forwarder is allowed to query.

use tauri::State;

use crate::Binary::Build::{DnsCommands::ForwardAllowList::ForwardAllowList, Scheme::DnsPort};

#[tauri::command]
pub fn dns_get_forward_allowlist(dns_port:State<DnsPort>) -> Result<ForwardAllowList, String> {

	if dns_port.0 == 0 {
		return Err("DNS server is not running".to_string());
	}

	Ok(ForwardAllowList { domains:vec!["update.land.playform.cloud.".to_string()] })
}
