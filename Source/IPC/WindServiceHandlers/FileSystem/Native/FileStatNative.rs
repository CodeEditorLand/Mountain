
//! Wire method `file:stat`. Returns VS Code's `IStat` shape via
//! `metadata_to_istat`. Uses `symlink_metadata` to avoid following
//! symlinks (matches Electron behaviour). Noise from benign ENOENTs on
//! known VS Code probe paths is squelched via `IsBenignEnoent` +
//! `DebugOnce`.

use serde_json::Value;

use crate::{
	IPC::{
		DevLog,
		WindServiceHandlers::Utilities::{
			MetadataEncoding::Fn as metadata_to_istat,
			PathExtraction::Fn as extract_path_from_arg,
		},
	},
	dev_log,
};

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(Arguments.get(0).ok_or("Missing file path")?)?;

	// Per-path stat emits at very high volume during workbench boot
	// (package.json / launch.json / settings.json probes from every
	// extension). Gate to `vfs-verbose`; the ENOENT path retains the
	// `vfs` tag so real misses still surface at the default level.
	if !DevLog::IsBenignEnoent::Fn(&Path) {
		dev_log!("vfs-verbose", "stat: {}", Path);
	}

	let Metadata = tokio::fs::symlink_metadata(&Path).await.map_err(|E| {
		if DevLog::IsBenignEnoent::Fn(&Path) {
			DevLog::DebugOnce::Fn(
				"vfs",
				&format!("stat-enoent:{}", Path),
				&format!("stat ENOENT (benign): {}", Path),
			);
		} else {
			dev_log!("vfs", "stat ENOENT: {}", Path);
		}
		format!("Failed to stat file: {} (path: {})", E, Path)
	})?;

	if !DevLog::IsBenignEnoent::Fn(&Path) {
		dev_log!("vfs-verbose", "stat OK: {} (dir={})", Path, Metadata.is_dir());
	}

	Ok(metadata_to_istat(&Metadata))
}
