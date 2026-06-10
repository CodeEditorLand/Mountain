//! Progress command dispatcher.

use serde_json::Value;

<<<<<<< HEAD
use crate::UI::{
=======
use crate::IPC::WindServiceHandlers::UI::{
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867
	ProgressBegin::Fn as ProgressBegin,
	ProgressEnd::Fn as ProgressEnd,
	ProgressReport::Fn as ProgressReport,
};

/// Dispatches progress commands.
///
/// Handled commands:
/// - `progress:begin`
/// - `progress:report`
/// - `progress:end`
pub async fn dispatch_progress(
	app_handle:&tauri::AppHandle,

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		"progress:begin" => ProgressBegin(app_handle.clone(), arguments).await,

		"progress:report" => ProgressReport(app_handle.clone(), arguments).await,

		"progress:end" => ProgressEnd(app_handle.clone(), arguments).await,

		_ => Err(format!("Unknown progress command: {}", command)),
	}
}
