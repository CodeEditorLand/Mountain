//! `dns_resolve` Tauri command - manual resolution helper used
//! by the diagnostic panel and by `dns_test_resolution`.
//!
//! `land.playform.cloud` zone names resolve to 127.0.0.1; allowlisted
//! external domains return a TEST-NET-1 placeholder; everything
//! else fails with `error="Domain not in forward allowlist"`.

use tauri::State;

use crate::Binary::Build::{DnsCommands::DnsResolutionResult::DnsResolutionResult, Scheme::DnsPort};

#[tauri::command]
pub fn dns_resolve(domain:String, dns_port:State<DnsPort>) -> Result<DnsResolutionResult, String> {
	if dns_port.0 == 0 {
		return Err("DNS server is not running".to_string());
	}

	if domain.ends_with("land.playform.cloud") || domain.ends_with("land.playform.cloud.") {
		return Ok(DnsResolutionResult {
			domain:domain.clone(),
			record_type:"A".to_string(),
			addresses:vec!["127.0.0.1".to_string()],
			ttl:3600,
			succeeded:true,
			error:None,
		});
	}

	let allowlist = vec!["update.land.playform.cloud."];

	let is_allowed = allowlist.iter().any(|d| {
		let test_domain = if domain.ends_with('.') { domain.clone() } else { format!("{}.", domain) };

		test_domain == *d || test_domain.ends_with(d)
	});

	if !is_allowed {
		return Ok(DnsResolutionResult {
			domain:domain.clone(),
			record_type:"A".to_string(),
			addresses:vec![],
			ttl:0,
			succeeded:false,
			error:Some("Domain not in forward allowlist".to_string()),
		});
	}

	Ok(DnsResolutionResult {
		domain:domain.clone(),
		record_type:"A".to_string(),
		addresses:vec!["192.0.2.1".to_string()],
		ttl:300,
		succeeded:true,
		error:None,
	})
}
