#![allow(non_snake_case)]

//! `tls_check_cert_status` Tauri command - parse a cached
//! cert's `valid_until` (RFC3339), compare against now, and
//! flag whether renewal is due (within
//! `CertificateManager::RENEWAL_THRESHOLD_DAYS`).

use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager};

use crate::{
	Binary::Build::{CertificateManager::CertificateManager, TlsCommands::CertificateStatus::CertificateStatus},
	dev_log,
};

#[tauri::command]
pub async fn tls_check_cert_status(app_handle:AppHandle, hostname:String) -> Result<CertificateStatus, String> {

	dev_log!("security", "checking certificate status for {}", hostname);

	let state = app_handle
		.try_state::<Arc<Mutex<CertificateManager>>>()
		.ok_or("Certificate manager not found")?;

	let cert_manager = state.clone();

	let manager = cert_manager.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;

	if let Some(cert_info) = manager.get_server_cert_info(&hostname) {

		let valid_until = chrono::DateTime::parse_from_rfc3339(&cert_info.valid_until)
			.map_err(|e| format!("Invalid certificate expiry time: {}", e))?
			.with_timezone(&chrono::Utc);

		let now = chrono::Utc::now();

		let days_until_expiry = (valid_until - now).num_days();

		let needs_renewal = days_until_expiry <= CertificateManager::RENEWAL_THRESHOLD_DAYS;

		Ok(CertificateStatus {
			exists:true,
			is_valid:now <= valid_until,
			days_until_expiry,
			needs_renewal,
			valid_until:cert_info.valid_until,
		})
	} else {

		Ok(CertificateStatus {
			exists:false,
			is_valid:false,
			days_until_expiry:0,
			needs_renewal:true,
			valid_until:String::new(),
		})
	}
}
