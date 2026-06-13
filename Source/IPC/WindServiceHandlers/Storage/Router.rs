//! Storage command router.
//!
//! Routes the six storage IPC commands to their handler functions.
//! Returns `None` for unrecognised commands so the dispatcher can
//! fall through to stub handlers or the generic prefix guard.

use std::sync::Arc;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;
use super::{
	StorageDelete::Fn as StorageDelete,
	StorageGet::Fn as StorageGet,
	StorageGetItems::Fn as StorageGetItems,
	StorageKeys::Fn as StorageKeys,
	StorageSet::Fn as StorageSet,
	StorageUpdateItems::Fn as StorageUpdateItems,
};

/// Routes storage commands.  Returns `Some(result)` for handled
/// commands, `None` otherwise.
pub(crate) async fn route(
	RunTime:Arc<ApplicationRunTime>,

	command:&str,

	Arguments:Vec<Value>,
) -> Option<Result<Value, String>> {
	match command {
		"storage:get" => Some(StorageGet(RunTime, Arguments).await),

		"storage:set" => Some(StorageSet(RunTime, Arguments).await),

		"storage:getItems" => Some(StorageGetItems(RunTime, Arguments).await),

		"storage:updateItems" => Some(StorageUpdateItems(RunTime, Arguments).await),

		"storage:delete" => Some(StorageDelete(RunTime, Arguments).await),

		"storage:keys" => Some(StorageKeys(RunTime).await),

		_ => None,
	}
}
