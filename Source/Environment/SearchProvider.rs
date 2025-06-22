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
	path::{Path, PathBuf},
	sync::{Arc, Mutex},
};

use Common::{Error::CommonError, Search::SearchProvider::SearchProvider};
use async_trait::async_trait;
use grep_regex::RegexMatcher;
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
	is_regex:Option<bool>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct TextMatch {
	preview:String,
	line_number:u64,
	// TODO: Add ranges: Vec<(u64, u64)> for highlighting matches
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct FileMatch {
	resource:String, // URI
	matches:Vec<TextMatch>,
}

#[derive(Clone)]
struct MatchSink {
	matches:Arc<Mutex<Vec<FileMatch>>>,
	current_file_path:Arc<Mutex<Option<PathBuf>>>,
}

impl Sink for MatchSink {
	type Error = std::io::Error;

	fn matched(&mut self, _searcher:&Searcher, mat:&SinkMatch<'_>) -> Result<bool, Self::Error> {
		let path_guard = self.current_file_path.lock().unwrap();
		let path = path_guard.as_ref().unwrap(); // Should be set by `begin`
		let mut matches_guard = self.matches.lock().unwrap();

		let preview = String::from_utf8_lossy(mat.lines().next().unwrap_or_default()).to_string();

		let file_uri = url::Url::from_file_path(&path).unwrap().to_string();

		if let Some(file_match) = matches_guard.iter_mut().find(|fm| fm.resource == file_uri) {
			file_match
				.matches
				.push(TextMatch { preview, line_number:mat.line_number().unwrap_or(0) });
		} else {
			matches_guard.push(FileMatch {
				resource:file_uri,
				matches:vec![TextMatch { preview, line_number:mat.line_number().unwrap_or(0) }],
			});
		}

		Ok(true)
	}

	fn begin(&mut self, _searcher:&Searcher, path:&Path) -> Result<bool, Self::Error> {
		*self.current_file_path.lock().unwrap() = Some(path.to_path_buf());
		Ok(true)
	}
}

#[async_trait]
impl SearchProvider for MountainEnvironment {
	async fn TextSearch(&self, QueryValue:Value, _OptionsValue:Value) -> Result<Value, CommonError> {
		let Query:TextSearchQuery = serde_json::from_value(QueryValue)?;
		info!("[SearchProvider] Performing text search for: {:?}", Query);

		let matcher = RegexMatcher::new_builder()
			.case_insensitive(!Query.is_case_sensitive.unwrap_or(false))
			.word(Query.is_word_match.unwrap_or(false))
			.build(&Query.pattern)
			.map_err(|e| CommonError::InvalidArgument { ArgumentName:"pattern".into(), Reason:e.to_string() })?;

		let searcher = Searcher::new();
		let sink = MatchSink {
			matches:Arc::new(Mutex::new(Vec::new())),
			current_file_path:Arc::new(Mutex::new(None)),
		};

		let folders = self.ApplicationState.WorkSpaceFolders.lock().unwrap().clone();
		if folders.is_empty() {
			warn!("[SearchProvider] No workspace folders to search in.");
			return Ok(json!([]));
		}

		for folder in folders {
			if let Ok(folder_path) = folder.URI.to_file_path() {
				let walker = WalkBuilder::new(folder_path).build();
				for result in walker {
					let Ok(entry) = result else { continue };
					if entry.file_type().map_or(false, |ft| ft.is_file()) {
						if let Err(e) = searcher.search_path(&matcher, entry.path(), sink.clone()) {
							warn!("[SearchProvider] Error searching path {}: {}", entry.path().display(), e);
						}
					}
				}
			}
		}

		let final_matches = sink.matches.lock().unwrap().clone();
		Ok(json!(final_matches))
	}
}
