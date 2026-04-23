#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method `file:stat`. Returns VS Code's `IStat` shape via
//! `metadata_to_istat`. Uses `symlink_metadata` to avoid following
//! symlinks (matches Electron behaviour). Noise from benign ENOENTs on
//! known VS Code probe paths is squelched via `IsBenignEnoent` +
//! `DebugOnce`.

use serde_json::Value;

use crate::IPC::WindServiceHandlers::Utilities::{
	MetadataEncoding::metadata_to_istat,
	PathExtraction::extract_path_from_arg,
};
use crate::{IPC::DevLog, dev_log};

pub async fn handle_file_stat_native(args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(args.get(0).ok_or("Missing file path")?)?;

	if !DevLog::IsBenignEnoent(&Path) {
		dev_log!("vfs", "stat: {}", Path);
	}

	let Metadata = tokio::fs::symlink_metadata(&Path).await.map_err(|E| {
		if DevLog::IsBenignEnoent(&Path) {
			DevLog::DebugOnce(
				"vfs",
				&format!("stat-enoent:{}", Path),
				&format!("stat ENOENT (benign): {}", Path),
			);
		} else {
			dev_log!("vfs", "stat ENOENT: {}", Path);
		}
		format!("Failed to stat file: {} (path: {})", E, Path)
	})?;

	if !DevLog::IsBenignEnoent(&Path) {
		dev_log!("vfs", "stat OK: {} (dir={})", Path, Metadata.is_dir());
	}
	Ok(metadata_to_istat(&Metadata))
}
