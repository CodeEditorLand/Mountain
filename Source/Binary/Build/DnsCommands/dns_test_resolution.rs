#![allow(non_snake_case)]

//! `dns_test_resolution` Tauri command - thin wrapper over
//! `dns_resolve` that flattens to a `bool` for automated
//! health-check loops.

use tauri::State;

use crate::Binary::Build::{DnsCommands::dns_resolve::dns_resolve, Scheme::DnsPort};

#[tauri::command]
pub fn dns_test_resolution(domain:String, dns_port:State<DnsPort>) -> Result<bool, String> {
	let result = dns_resolve(domain, dns_port)?;

	Ok(result.succeeded)
}
