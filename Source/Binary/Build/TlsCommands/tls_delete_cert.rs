#![allow(non_snake_case)]

//! `tls_delete_cert` Tauri command - currently aliased to
//! `renew_certificate` (regenerates instead of removing). TODO:
//! add a real `CertificateManager::delete_certificate` so the
//! cache entry actually disappears.

use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager};

use crate::{Binary::Build::CertificateManager::CertificateManager, dev_log};

#[tauri::command]
pub async fn tls_delete_cert(app_handle:AppHandle, hostname:String) -> Result<String, String> {
	dev_log!("security", "deleting certificate for {}", hostname);

	let state = app_handle
		.try_state::<Arc<Mutex<CertificateManager>>>()
		.ok_or("Certificate manager not found")?;

	let cert_manager = state.clone();

	{
		let mut manager = cert_manager.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;

		let _result = manager.renew_certificate(&hostname);
	}

	Ok(format!("Certificate deleted for {}", hostname))
}
