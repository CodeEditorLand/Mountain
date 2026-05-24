//! `TlsGetCaCert` Tauri command - returns the CA certificate
//! PEM so the webview can pin it or Install it into the system
//! trust store.

use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager};

use crate::{Binary::Build::CertificateManager::CertificateManager, dev_log};

#[tauri::command]
pub async fn Fn(app_handle:AppHandle) -> Result<String, String> {
	dev_log!("security", "getting CA certificate");

	let state = app_handle
		.try_state::<Arc<Mutex<CertificateManager>>>()
		.ok_or("Certificate manager not found")?;

	let cert_manager = state.clone();

	let manager = cert_manager.lock().map_err(|E| format!("Failed to acquire lock: {}", e))?;

	let cert_pem = manager.GetCaCertPem().ok_or("CA certificate not initialized")?;

	String::from_utf8(cert_pem).map_err(|E| format!("Invalid certificate UTF-8: {}", e))
}
