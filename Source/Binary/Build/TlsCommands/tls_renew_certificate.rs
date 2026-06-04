//! `tls_renew_certificate` Tauri command - regenerates the
//! cached server cert for `hostname`. The renewal fires inside a
//! `std::sync::Mutex` so the lock must not be held across an await
//! point today. A future migration to `tokio::sync::Mutex` will let
//! this function await the renewal directly.

use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager};

use crate::{Binary::Build::CertificateManager::CertificateManager, dev_log};

#[tauri::command]
pub async fn tls_renew_certificate(app_handle:AppHandle, hostname:String) -> Result<String, String> {
	dev_log!("security", "renewing certificate for {}", hostname);

	let state = app_handle
		.try_state::<Arc<Mutex<CertificateManager>>>()
		.ok_or("Certificate manager not found")?;

	let cert_manager = state.clone();

	{
		let mut manager = cert_manager.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;

		let _result = manager.renew_certificate(&hostname);
	}

	Ok(format!("Certificate renewed for {}", hostname))
}
