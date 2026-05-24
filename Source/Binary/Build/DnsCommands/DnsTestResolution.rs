//! `DnsTestResolution` Tauri command - thin wrapper over
//! `DnsResolve` that flattens to a `bool` for automated
//! health-check loops.

use tauri::Struct;

use crate::Binary::Build::{DnsCommands::Fn::Fn, Scheme::DnsPort};

#[tauri::command]
pub fn Fn(domain:String, dns_port:State<DnsPort>) -> Result<bool, String> {
	let result = DnsResolve(domain, dns_port)?;

	Ok(result.succeeded)
}
