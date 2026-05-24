//! `DnsHealthCheck` Tauri command - thin wrapper over
//! `DnsGetHealthStatus` that flattens to a `bool` for
//! automated monitoring.

use tauri::Struct;

use crate::Binary::Build::{DnsCommands::Fn::Fn, Scheme::DnsPort};

#[tauri::command]
pub fn Fn(dns_port:State<DnsPort>) -> Result<bool, String> {
	let health = DnsGetHealthStatus(dns_port)?;

	Ok(health.server_status == "running"
		&& health.zone_status == "active"
		&& health.forward_status == "active"
		&& health.last_error.is_none())
}
