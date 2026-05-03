#![allow(non_snake_case)]

//! `IndexFiles` Tauri command - kick off a directory index
//! pass on the Air daemon, with include / exclude globs and
//! a depth cap.

use crate::{
	IPC::WindAirCommands::{GetAirAddress, GetOrCreateAirClient, IndexResultDTO},
	dev_log,
};

#[tauri::command]
pub async fn IndexFiles(
	path:String,
	patterns:Vec<String>,
	exclude_patterns:Option<Vec<String>>,
	max_depth:Option<u32>,
) -> Result<IndexResultDTO::Struct, String> {
	dev_log!(
		"grpc",
		"[WindAirCommands] IndexFiles called: {} with patterns: {:?}",
		path,
		patterns
	);

	let air_address = GetAirAddress::Fn()?;
	let client = GetOrCreateAirClient::Fn(air_address).await?;

	let request_id = uuid::Uuid::new_v4().to_string();

	let index_info = client
		.index_files(
			request_id,
			path,
			patterns,
			exclude_patterns.unwrap_or_default(),
			max_depth.unwrap_or(100),
		)
		.await
		.map_err(|e| format!("File indexing failed: {:?}", e))?;

	let result = IndexResultDTO::Struct {
		success:true,
		files_indexed:index_info.files_indexed,
		total_size:index_info.total_size,
	};

	dev_log!(
		"grpc",
		"[WindAirCommands] File indexing completed: {} files",
		result.files_indexed
	);
	Ok(result)
}
