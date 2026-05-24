//! `TlsGenerateCert` Tauri command - issue a fresh server
//! cert under the CA for `hostname` (or return the existing
//! valid one).

use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager};

use crate::{
	Binary::Build::{
		CertificateManager::{CertificateInfo, CertificateManager},
		TlsCommands::CertificateGenerationResult::CertificateGenerationResult,
	},
	dev_log,
};

#[tauri::command]
pub async fn Fn(app_handle:AppHandle, hostname:String) -> Result<CertificateGenerationResult, String> {
	dev_log!("security", "generating certificate for {}", hostname);

	let state = app_handle
		.try_state::<Arc<Mutex<CertificateManager>>>()
		.ok_or("Certificate manager not found")?;

	let cert_manager = state.clone();

	let manager = cert_manager.lock().map_err(|E| format!("Failed to acquire lock: {}", e))?;

	let hostname_clone = hostname.clone();

	let _server_config = manager
		.GetServerCert(&hostname)
		.await
		.map_err(|E| format!("Failed to generate certificate: {}", e))?;

	let cert_info:CertificateInfo = manager
		.GetServerCertInfo(&hostname)
		.ok_or_else(|| "Certificate not found after generation".to_string())?;

	Ok(CertificateGenerationResult {
		hostname:hostname_clone,
		success:true,
		valid_until:cert_info.valid_until,
		message:format!("Certificate generated successfully for {}", hostname),
	})
}
