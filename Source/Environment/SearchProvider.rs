// File: Mountain/Source/Environment/SearchProvider.rs
// Role: Implements the `SearchProvider` trait for `MountainEnvironment`.
// Responsibilities:
//   - Perform workspace-wide text searches using `grep-searcher` (the `ripgrep`
//     library).
//   - Respect workspace folders and standard ignore files (`.gitignore`).
//   - Collect and format search results into a DTO suitable for the frontend.

//! # SearchProvider Implementation
//!
//! Implements the `SearchProvider` trait using the `grep-searcher` crate, which
//! is a library for the `ripgrep` search tool.

use std::{
	io,
	path::PathBuf,
	sync::{Arc, Mutex},
};

use Common::{Error::CommonError::CommonError, Search::SearchProvider::SearchProvider};
use async_trait::async_trait;
use grep_regex::RegexMatcherBuilder;
use grep_searcher::{Searcher, Sink, SinkMatch};
use ignore::WalkBuilder;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::MountainEnvironment::MountainEnvironment;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TextSearchQuery {
	pattern:String,

	is_case_sensitive:Option<bool>,

	is_word_match:Option<bool>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct TextMatch {
	preview:String,

	line_number:u64,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct FileMatch {
	// URI
	resource:String,

	matches:Vec<TextMatch>,
}

// This Sink is designed to be created for each file.
// It holds a reference to the central results vector and the path of the file
// it's searching.
struct PerFileSink {
	path:PathBuf,

	results:Arc<Mutex<Vec<FileMatch>>>,
}

impl Sink for PerFileSink {
	type Error = io::Error;

	fn matched(&mut self, _searcher:&Searcher, mat:&SinkMatch<'_>) -> Result<bool, Self::Error> {
		let mut results_guard = self.results.lock().unwrap();

		let preview = String::from_utf8_lossy(mat.bytes()).to_string();

		let line_number = mat.line_number().unwrap_or(0);

		// Since this sink is per-file, we know `self.path` is correct.
		let file_uri = url::Url::from_file_path(&self.path).unwrap().to_string();

		// Find the entry for our file, or create it if it's the first match.
		if let Some(file_match) = results_guard.iter_mut().find(|fm| fm.resource == file_uri) {
			file_match.matches.push(TextMatch { preview, line_number });
		} else {
			results_guard.push(FileMatch { resource:file_uri, matches:vec![TextMatch { preview, line_number }] });
		}

		// Continue searching
		Ok(true)
	}
}

#[async_trait]
impl SearchProvider for MountainEnvironment {
	async fn TextSearch(&self, QueryValue:Value, _OptionsValue:Value) -> Result<Value, CommonError> {
		let Query:TextSearchQuery = serde_json::from_value(QueryValue)?;

		info!("[SearchProvider] Performing text search for: {:?}", Query);

		let mut builder = RegexMatcherBuilder::new();

		builder
			.case_insensitive(!Query.is_case_sensitive.unwrap_or(false))
			.word(Query.is_word_match.unwrap_or(false));

		let matcher = builder
			.build(&Query.pattern)
			.map_err(|e| CommonError::InvalidArgument { ArgumentName:"pattern".into(), Reason:e.to_string() })?;

		let all_matches = Arc::new(Mutex::new(Vec::<FileMatch>::new()));

		let folders_guard = self.ApplicationState.WorkSpaceFolders.lock().unwrap();

		let folders = folders_guard.clone();

		drop(folders_guard);

		if folders.is_empty() {
			warn!("[SearchProvider] No workspace folders to search in.");

			return Ok(json!([]));
		}

		for folder in folders {
			if let Ok(folder_path) = folder.URI.to_file_path() {
				// Use a parallel walker for better performance.
				let walker = WalkBuilder::new(folder_path).build_parallel();

				// The `search_parallel` method is not available on `Searcher`.
				// We must process entries from the walker and call `search_path` individually.
				walker.run(|| {
					let mut searcher = Searcher::new();

					let matcher = matcher.clone();

					let all_matches = all_matches.clone();

					Box::new(move |entry_result| {
						if let Ok(entry) = entry_result {
							if entry.file_type().map_or(false, |ft| ft.is_file()) {
								// For each file, create a new sink that knows its path.
								let sink = PerFileSink { path:entry.path().to_path_buf(), results:all_matches.clone() };

								if let Err(e) = searcher.search_path(&matcher, entry.path(), sink) {
									warn!("[SearchProvider] Error searching path {}: {}", entry.path().display(), e);
								}
							}
						}

						ignore::WalkState::Continue
					})
				});
			}
		}

		let final_matches = all_matches.lock().unwrap().clone();

		Ok(json!(final_matches))
	}
}
