#![allow(non_snake_case)]

//! `tls_renew_certificate` Tauri command - regenerates the
//! cached server cert for `hostname`.
//!
//! TODO: the inner `Mutex` should become `tokio::sync::Mutex`
//! so the lock can be held across `await`; the renewal call is
//! currently fire-and-forget.

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
