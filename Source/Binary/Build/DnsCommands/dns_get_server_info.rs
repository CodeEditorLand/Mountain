
//! `dns_get_server_info` Tauri command - returns port, running
//! flag, and startup time.

use tauri::State;

use crate::Binary::Build::{
	DnsCommands::{DnsServerInfo::DnsServerInfo, StartupTime},
	Scheme::DnsPort,
};

#[tauri::command]
pub fn dns_get_server_info(dns_port:State<DnsPort>) -> Result<DnsServerInfo, String> {
	let port = dns_port.0;

	let is_running = port > 0;

	let startup_time = StartupTime::Get();

	Ok(DnsServerInfo { port, is_running, startup_time })
}
