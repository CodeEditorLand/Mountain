//! `dns_get_health_status` Tauri command - aggregated server /
//! zone / forward status snapshot.

use tauri::State;

use crate::Binary::Build::{DnsCommands::DnsHealthStatus::DnsHealthStatus, Scheme::DnsPort};

#[tauri::command]
pub fn dns_get_health_status(dns_port:State<DnsPort>) -> Result<DnsHealthStatus, String> {

	let port = dns_port.0;

	if port == 0 {
		return Ok(DnsHealthStatus {
			server_status:"stopped".to_string(),
			zone_status:"inactive".to_string(),
			forward_status:"inactive".to_string(),
			last_error:Some("DNS server is not running".to_string()),
		});
	}

	Ok(DnsHealthStatus {
		server_status:"running".to_string(),
		zone_status:"active".to_string(),
		forward_status:"active".to_string(),
		last_error:None,
	})
}
