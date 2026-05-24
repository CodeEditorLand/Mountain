//! `TlsGetServerCertInfo` Tauri command - certificate info
//! lookup for one hostname (returns `None` when no cached cert).

use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager};

use crate::{
	Binary::Build::CertificateManager::{CertificateInfo, CertificateManager},
	dev_log,
};

#[tauri::command]
pub async fn Fn(app_handle:AppHandle, hostname:String) -> Result<Option<CertificateInfo>, String> {
	dev_log!("security", "getting server cert info for {}", hostname);

	let state = app_handle
		.try_state::<Arc<Mutex<CertificateManager>>>()
		.ok_or("Certificate manager not found")?;

	let cert_manager = state.clone();

	let manager = cert_manager.lock().map_err(|E| format!("Failed to acquire lock: {}", e))?;

	Ok(manager.GetServerCertInfo(&hostname))
}
