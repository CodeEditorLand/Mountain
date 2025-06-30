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

#![allow(non_snake_case, non_camel_case_types)]

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

use super::{MountainEnvironment::MountainEnvironment, Utility};

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

// This Sink is designed to be created for each file. It holds a reference to
// the central results vector and the path of the file it's searching.
struct PerFileSink {
	path:PathBuf,

	results:Arc<Mutex<Vec<FileMatch>>>,
}

impl Sink for PerFileSink {
	type Error = io::Error;

	fn matched(&mut self, _Searcher:&Searcher, Mat:&SinkMatch<'_>) -> Result<bool, Self::Error> {
		let mut ResultsGuard = self
			.results
			.lock()
			.map_err(|Error| io::Error::new(io::ErrorKind::Other, Error.to_string()))?;

		let Preview = String::from_utf8_lossy(Mat.bytes()).to_string();

		let LineNumber = Mat.line_number().unwrap_or(0);

		// Since this sink is per-file, we know `self.path` is correct.
		let FileURI = url::Url::from_file_path(&self.path)
			.map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Could not convert path to URL"))?
			.to_string();

		// Find the entry for our file, or create it if it's the first match.
		if let Some(FileMatch) = ResultsGuard.iter_mut().find(|fm| fm.resource == FileURI) {
			FileMatch.matches.push(TextMatch { preview:Preview, line_number:LineNumber });
		} else {
			ResultsGuard.push(FileMatch {
				resource:FileURI,

				matches:vec![TextMatch { preview:Preview, line_number:LineNumber }],
			});
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

		let mut Builder = RegexMatcherBuilder::new();

		Builder
			.case_insensitive(!Query.is_case_sensitive.unwrap_or(false))
			.word(Query.is_word_match.unwrap_or(false));

		let Matcher = Builder.build(&Query.pattern).map_err(|Error| {
			CommonError::InvalidArgument { ArgumentName:"pattern".into(), Reason:Error.to_string() }
		})?;

		let AllMatches = Arc::new(Mutex::new(Vec::<FileMatch>::new()));

		let Folders = self
			.ApplicationState
			.WorkSpaceFolders
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.clone();

		if Folders.is_empty() {
			warn!("[SearchProvider] No workspace folders to search in.");

			return Ok(json!([]));
		}

		for Folder in Folders {
			if let Ok(FolderPath) = Folder.URI.to_file_path() {
				// Use a parallel walker for better performance.
				let Walker = WalkBuilder::new(FolderPath).build_parallel();

				// The `search_parallel` method is not available on `Searcher`. We must process
				// entries from the walker and call `search_path` individually.
				Walker.run(|| {
					let mut Searcher = Searcher::new();

					let Matcher = Matcher.clone();

					let AllMatches = AllMatches.clone();

					Box::new(move |EntryResult| {
						if let Ok(Entry) = EntryResult {
							if Entry.file_type().map_or(false, |ft| ft.is_file()) {
								// For each file, create a new sink that knows its path.
								let Sink = PerFileSink { path:Entry.path().to_path_buf(), results:AllMatches.clone() };

								if let Err(Error) = Searcher.search_path(&Matcher, Entry.path(), Sink) {
									warn!(
										"[SearchProvider] Error searching path {}: {}",
										Entry.path().display(),
										Error
									);
								}
							}
						}

						ignore::WalkState::Continue
					})
				});
			}
		}

		let FinalMatches = AllMatches
			.lock()
			.map_err(|Error| CommonError::StateLockPoisoned { Context:Error.to_string() })?
			.clone();

		Ok(json!(FinalMatches))
	}
}
