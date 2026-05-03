#![allow(non_snake_case)]

//! `dns_health_check` Tauri command - thin wrapper over
//! `dns_get_health_status` that flattens to a `bool` for
//! automated monitoring.

use tauri::State;

use crate::Binary::Build::{DnsCommands::dns_get_health_status::dns_get_health_status, Scheme::DnsPort};

#[tauri::command]
pub fn dns_health_check(dns_port:State<DnsPort>) -> Result<bool, String> {
	let health = dns_get_health_status(dns_port)?;

	Ok(health.server_status == "running"
		&& health.zone_status == "active"
		&& health.forward_status == "active"
		&& health.last_error.is_none())
}
