//! WorkingCopy dispatcher.

use serde_json::Value;

use crate::UI::{
	WorkingCopyGetAllDirty::Fn as WorkingCopyGetAllDirty,
	WorkingCopyGetDirtyCount::Fn as WorkingCopyGetDirtyCount,
	WorkingCopyIsDirty::Fn as WorkingCopyIsDirty,
	WorkingCopySetDirty::Fn as WorkingCopySetDirty,
};

/// Dispatches working copy commands.
pub async fn dispatch_working_copy(
	runtime:&crate::RunTime::ApplicationRunTime::ApplicationRunTime,

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		"workingCopy:isDirty" => WorkingCopyIsDirty(runtime.clone(), arguments).await,

		"workingCopy:setDirty" => WorkingCopySetDirty(runtime.clone(), arguments).await,

		"workingCopy:getAllDirty" => WorkingCopyGetAllDirty(runtime.clone()).await,

		"workingCopy:getDirtyCount" => WorkingCopyGetDirtyCount(runtime.clone()).await,

		_ => Err(format!("Unknown working copy command: {}", command)),
	}
}
