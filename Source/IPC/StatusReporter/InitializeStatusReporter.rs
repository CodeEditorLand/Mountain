
//! Bootstrap helper - construct the `StatusReporter::Reporter`
//! and stash a clone in the app's Tauri state so the
//! `mountain_*` Tauri commands can `try_state::<Reporter>()`.

use std::sync::Arc;

use tauri::Manager;

use crate::{
	IPC::StatusReporter::Reporter::Struct as Reporter,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

pub fn initialize_status_reporter(app_handle:&tauri::AppHandle, runtime:Arc<ApplicationRunTime>) -> Result<(), String> {
	dev_log!("lifecycle", "Initializing status reporter");

	let reporter = Reporter::new(runtime);

	app_handle.manage(reporter.clone_reporter());

	Ok(())
}
