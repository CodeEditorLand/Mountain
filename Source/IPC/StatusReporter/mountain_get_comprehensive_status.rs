
//! `mountain_get_comprehensive_status` Tauri command -
//! assembles a `ComprehensiveStatusReport::Struct` (basic
//! status + performance metrics + health) in one call.

use std::time::SystemTime;

use tauri::Manager;

use crate::{
	IPC::StatusReporter::{
		ComprehensiveStatusReport::Struct as ComprehensiveStatusReport,
		Reporter::Struct as Reporter,
	},
	dev_log,
};

#[tauri::command]
pub async fn mountain_get_comprehensive_status(
	app_handle:tauri::AppHandle,
) -> Result<ComprehensiveStatusReport, String> {
	dev_log!("lifecycle", "Tauri command: get_comprehensive_status");

	if let Some(reporter) = app_handle.try_state::<Reporter>() {
		let basic_status = reporter.generate_status_report().await?;

		let performance_metrics = reporter.get_performance_metrics()?;

		let health_status = reporter.get_health_status()?;

		Ok(ComprehensiveStatusReport {
			basic_status,
			performance_metrics,
			health_status,
			timestamp:SystemTime::now()
				.duration_since(SystemTime::UNIX_EPOCH)
				.unwrap_or_default()
				.as_millis() as u64,
		})
	} else {
		Err("StatusReporter not found in application state".to_string())
	}
}
