//! `file:close` - close an fd returned by `file:open` and free the entry.
//!
//! Arguments\[0\] = integer fd (as returned by FileOpenFd).
//! Silently ignores unknown fds (VS Code may call close on an already-
//! closed fd during workbench teardown).

use serde_json::Value;

use crate::{
	IPC::WindServiceHandlers::{FileSystem::Native::FileOpenFd::GetFdTable, Utilities::JsonValueHelpers::arg_u64},
	dev_log,
};

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let Fd = arg_u64(&Arguments, 0) as u32;

	if Fd == 0 {
		return Ok(Value::Null);
	}

	if let Ok(mut Table) = GetFdTable().Files.lock() {
		let Removed = Table.remove(&Fd).is_some();

		dev_log!("vfs", "file:close fd={} removed={}", Fd, Removed);
	}

	Ok(Value::Null)
}
