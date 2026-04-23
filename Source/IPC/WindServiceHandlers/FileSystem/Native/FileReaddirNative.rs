#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method `file:readdir`. Returns `[[name, fileType]]` matching
//! VS Code's `ReadDirResult` (`FileType`: File=1, Directory=2,
//! SymbolicLink=64 - combined via bitflags upstream, but the readdir
//! callers only care about the per-entry value).

use serde_json::{Value, json};

use crate::{IPC::WindServiceHandlers::Utilities::PathExtraction::extract_path_from_arg, dev_log};

pub async fn handle_file_readdir_native(args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(args.get(0).ok_or("Missing directory path")?)?;

	dev_log!("vfs", "readdir: {}", Path);

	let mut Entries = tokio::fs::read_dir(&Path)
		.await
		.map_err(|E| format!("Failed to readdir: {} ({})", Path, E))?;

	let mut Result = Vec::new();

	while let Some(Entry) = Entries.next_entry().await.map_err(|E| E.to_string())? {
		let Name = Entry.file_name().to_string_lossy().to_string();
		let FileType = Entry.file_type().await.map_err(|E| E.to_string())?;

		let TypeValue = if FileType.is_symlink() {
			64
		} else if FileType.is_dir() {
			2
		} else {
			1
		};

		Result.push(json!([Name, TypeValue]));
	}

	Ok(json!(Result))
}
