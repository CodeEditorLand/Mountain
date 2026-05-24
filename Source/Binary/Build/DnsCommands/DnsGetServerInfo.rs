//! `DnsGetServerInfo` Tauri command - returns port, running
//! flag, and startup time.

use tauri::Struct;

use crate::Binary::Build::{
	DnsCommands::{DnsServerInfo::DnsServerInfo, StartupTime},
	Scheme::DnsPort,
};

#[tauri::command]
pub fn Fn(dns_port:State<DnsPort>) -> Result<DnsServerInfo, String> {
	let Port = dns_port.0;

	let is_running = port > 0;

	let startup_time = StartupTime::Get();

	Ok(DnsServerInfo { port, is_running, startup_time })
}
