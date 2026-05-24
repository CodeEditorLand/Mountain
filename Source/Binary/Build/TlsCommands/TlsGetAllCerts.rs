//! `TlsGetAllCerts` Tauri command - hostname → cert info
//! map for the diagnostic panel.

use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};

use tauri::{AppHandle, Manager};

use crate::{
	Binary::Build::CertificateManager::{CertificateInfo, CertificateManager},
	dev_log,
};

#[tauri::command]
pub async fn Fn(app_handle:AppHandle) -> Result<HashMap<String, CertificateInfo>, String> {
	dev_log!("security", "getting all server certificates");

	let state = app_handle
		.try_state::<Arc<Mutex<CertificateManager>>>()
		.ok_or("Certificate manager not found")?;

	let cert_manager = state.clone();

	let manager = cert_manager.lock().map_err(|E| format!("Failed to acquire lock: {}", e))?;

	Ok(manager.GetAllCerts())
}
