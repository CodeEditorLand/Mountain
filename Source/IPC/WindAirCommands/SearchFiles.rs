#![allow(non_snake_case)]

//! `SearchFiles` Tauri command - query Air's full-text index
//! and shape hits into `SearchResultsDTO::Struct`.

use crate::{
	IPC::WindAirCommands::{FileResultDTO, GetAirAddress, GetOrCreateAirClient, SearchResultsDTO},
	dev_log,
};

#[tauri::command]
pub async fn SearchFiles(
	query:String,
	file_patterns:Vec<String>,
	max_results:Option<u32>,
) -> Result<SearchResultsDTO::Struct, String> {
	dev_log!(
		"grpc",
		"[WindAirCommands] SearchFiles called: query={}, patterns={:?}",
		query,
		file_patterns
	);

	let air_address = GetAirAddress::Fn()?;
	let client = GetOrCreateAirClient::Fn(air_address).await?;

	let request_id = uuid::Uuid::new_v4().to_string();
	let max_results_count = max_results.unwrap_or(100);

	let search_results = client
		.search_files(
			request_id,
			query,
			file_patterns.first().map(|s| s.as_str()).unwrap_or("").to_string(),
			max_results_count,
		)
		.await
		.map_err(|e| format!("File search failed: {:?}", e))?;

	let results:Vec<FileResultDTO::Struct> = search_results
		.into_iter()
		.map(|r| {
			FileResultDTO::Struct {
				path:r.path,
				size:r.size,
				line:Some(r.line_number),
				content:Some(r.match_preview),
			}
		})
		.collect();

	let total_results = results.len() as u32;
	let result = SearchResultsDTO::Struct { results, total_results };

	dev_log!(
		"grpc",
		"[WindAirCommands] File search completed: {} results",
		result.total_results
	);
	Ok(result)
}
