#![allow(non_snake_case)]

//! Tauri command - generic shared-process service health probe.
//! Hard-coded readiness map for the three currently-shipped services
//! (storage / update / search); unknown services return `false`.

#[tauri::command]
pub async fn shared_process_service_health(service:String) -> Result<bool, String> {
	match service.as_str() {
		"storage" | "update" | "search" => Ok(true),

		_ => Ok(false),
	}
}
