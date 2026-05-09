#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Converts `std::fs::Metadata` to VS Code's `IStat` wire shape. The
//! `FileType` bits are VS Code's enum (File=1, Directory=2,
//! SymbolicLink=64); the readdir atoms also emit these values.

use serde_json::{Value, json};

pub fn metadata_to_istat(Metadata:&std::fs::Metadata) -> Value {

	let FileType = if Metadata.is_symlink() {

		64
	} else if Metadata.is_dir() {

		2
	} else {

		1
	};

	let Size = Metadata.len();

	let Mtime = Metadata
		.modified()
		.ok()
		.and_then(|T| T.duration_since(std::time::UNIX_EPOCH).ok())
		.map(|D| D.as_millis() as u64)
		.unwrap_or(0);

	let Ctime = Metadata
		.created()
		.ok()
		.and_then(|T| T.duration_since(std::time::UNIX_EPOCH).ok())
		.map(|D| D.as_millis() as u64)
		.unwrap_or(Mtime);

	json!({
		"type": FileType,
		"size": Size,
		"mtime": Mtime,
		"ctime": Ctime
	})
}
