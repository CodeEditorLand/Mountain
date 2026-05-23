//! `tls_initialize` Tauri command - loads the CA from the
//! keyring or generates a fresh one. Must run before any other
//! TLS command on this app handle.

use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager};

use crate::{Binary::Build::CertificateManager::CertificateManager, dev_log};

#[tauri::command]
pub async fn tls_initialize(app_handle:AppHandle) -> Result<String, String> {
	dev_log!("security", "TLS certificate manager initializing");

	let state = app_handle
		.try_state::<Arc<Mutex<CertificateManager>>>()
		.ok_or("Certificate manager not initialized in app state")?;

	let cert_manager = state.clone();

	let mut manager = cert_manager.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;

	manager
		.initialize_ca()
		.await
		.map_err(|e| format!("Failed to initialize CA: {}", e))?;

	dev_log!("security", "TLS certificate manager initialized");

	Ok("TLS certificate manager initialized".to_string())
}
