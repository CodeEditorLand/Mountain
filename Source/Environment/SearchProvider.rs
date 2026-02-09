//! # SearchProvider (Environment)
//!
//! Implements the `SearchProvider` trait for `MountainEnvironment`, providing
//! text search capabilities across files and content within the workspace.
//!
//! ## RESPONSIBILITIES
//!
//! ### 1. Search Execution
//! - Search for text patterns in files using glob patterns
//! - Support regular expression search
//! - Search file contents and/or file names
//! - Handle large result sets efficiently
//!
//! ### 2. Search Results
//! - Return structured search results with matches
//! - Include file URI, line number, column, and matching text
//! - Support paging and result limiting
//! - Sort results by relevance or file order
//!
//! ### 3. Search Configuration
//! - Respect workspace file exclusion patterns (.gitignore)
//! - Honor file size limits for search
//! - Support case-sensitive and whole-word matching
//! - Handle symbolic links appropriately
//!
//! ### 4. Search Cancellation
//! - Support cancellation of long-running searches
//! - Clean up resources on cancellation
//! - Provide progress feedback (optional)
//!
//! ## ARCHITECTURAL ROLE
//!
//! SearchProvider is the **workspace search engine**:
//!
//! ```text
//! Search Request ──► SearchProvider ──► FileSystem Scan ──► Results
//! ```
//!
//! ### Position in Mountain
//! - `Environment` module: Search capability provider
//! - Implements `CommonLibrary::Search::SearchProvider` trait
//! - Accessible via `Environment.Require<dyn SearchProvider>()`
//!
//! ### Search Types Supported
//! - **Text search**: Find files containing text pattern
//! - **File search**: Find files by name/glob pattern
//! - **Replace**: (Future) Search and replace operations
//! - **Context search**: (Future) Search with surrounding context
//!
//! ### Dependencies
//! - `FileSystemReader`: Read file contents for searching
//! - `WorkspaceProvider`: Get workspace folders to search
//! - `Log`: Search progress and errors
//!
//! ### Dependents
//! - Search UI panel: User-initiated searches
//! - Find/Replace dialogs: In-editor search
//! - Grep-like command-line operations
//! - Code navigation (symbol search)
//!
//! ## SEARCH PROCESS
//!
//! 1. **File Discovery**: Walk workspace directories, respecting exclusions
//! 2. **File Filtering**: Match filenames against include/exclude patterns
//! 3. **Content Search**: For each file, search for pattern in content
//! 4. **Match Collection**: Record matches with position information
//! 5. **Result Formatting**: Return structured search results
//!
//! ## PERFORMANCE CONSIDERATIONS
//!
//! - Search is I/O bound; consider async and parallel processing
//! - Large workspaces may have thousands of files
//! - Use file size limits to prevent memory exhaustion
//! - Implement result paging for UI responsiveness
//! - Consider background search indexing for faster repeated searches
//!
//! ## ERROR HANDLING
//!
//! - Permission denied: Skip file, log warning
//! - File not found: Skip file (may have been deleted)
//! - Encoding errors: Try default encoding, skip on failure
//! - Search cancelled: Stop immediately, return partial results
//!
//! ## VS CODE REFERENCE
//!
//! Patterns from VS Code:
//! - `vs/workbench/contrib/search/browser/searchWidget.ts` - Search UI
//! - `vs/platform/search/common/search.ts` - Search service API
//! - `vs/platform/search/common/fileSearch.ts` - File system search
//!
//! ## TODO
//!
//! - [ ] Implement file content indexing for faster searches
//! - [ ] Add regular expression support with PCRE or regex engine
//! - [ ] Support search result paging and streaming
//! - [ ] Add search cancellation with proper cleanup
//! - [ ] Implement search result highlighting in UI
//! - [ ] Support search in compressed/archive files
//! - [ ] Add search across multiple workspaces
//! - [ ] Implement search history and persistence
//! - [ ] Add search filters (by language, by file size, etc.)
//! - [ ] Support search templates and saved searches
//! - [ ] Implement search result grouping (by folder, by file)
//! - [ ] Add search performance metrics and logging
//! - [ ] Support search result export (to file, clipboard)
//!
//! ## MODULE CONTENTS
//!
//! - [`SearchProvider`]: Main struct implementing the trait
//! - Search execution methods
//! - File walking and filtering logic
//! - Match extraction and formatting
//! - Search cancellation support

// Responsibilities:
//   - Perform workspace-wide text searches using `grep-searcher` (the `ripgrep` library).
//   - Respect workspace folders and standard ignore files (`.gitignore`).
//   - Collect and format search results into a DTO suitable for the frontend.
//   - Support regex patterns and case-sensitive/insensitive searches.
//   - Implement word-boundary matching.
//   - Optimize for performance with parallel file walking.
//   - Handle large files efficiently with memory-efficient streaming.
//   - Support incremental search with result pagination.
//   - Provide search statistics (matches count, files searched).
//   - Handle search cancellation gracefully.
//
// TODOs:
//   - Implement result pagination for large result sets
//   - Add search cancellation via CancellationToken
//   - Support include/exclude file patterns
//   - Implement context lines for matches (before/after)
//   - Add file type filtering (e.g., search only in certain extensions)
//   - Implement replacement/match highlighting in results
//   - Add search progress reporting
//   - Support search across multiple workspace folders independently
//   - Implement search caching for repeated searches
//   - Add regex capture groups support
//   - Implement search history and recent searches
//   - Support search result export
//   - Add search performance metrics and optimization
//   - Implement search result deduplication
//   - Support glob patterns for file matching
//   - Add search result ranking and sorting
//   - Implement binary file handling (skip or search)
//   - Support symbolic link following
//   - Add max file size limit to avoid memory issues
//   - Implement search timeout
//   - Support search in hidden files
//   - Add line and column number precision
//   - Implement multi-line regex search
//
// Inspired by VSCode's search service which:
// - Uses ripgrep for high-performance text search
// - Supports complex regex patterns and modifiers
// - Provides context lines for matches
// - Handles large directories efficiently
// - Supports file and directory exclusions
// - Provides incremental search results
// - Handles search cancellation gracefully
//! # SearchProvider Implementation
//!
//! Implements the `SearchProvider` trait using the `grep-searcher` crate, which
//! is a library for the `ripgrep` search tool.
//!
//! ## Search Architecture
//!
//! The search implementation uses a multi-threaded approach:
//!
//! 1. **Pattern Compilation**: Regex pattern is compiled with modifiers
//! 2. **Parallel Walking**: Files in workspace are walked in parallel
//! 3. **Per-File Search**: Each file is searched individually using a sink
//!    pattern
//! 4. **Result Aggregation**: Matches are collected in a shared thread-safe
//!    vector
//!
//! ## Search Features
//!
//! - **Case Sensitivity**: Controlled by `is_case_sensitive` option
//! - **Word Matching**: Controlled by `is_word_match` option
//! - **Regex Support**: Full regex pattern matching via `grep-regex`
//! - **Ignore Files**: Respects `.gitignore`, `.ignore`, and other ignore files
//! - **Parallel Search**: Uses `WalkBuilder::build_parallel()` for performance
//! - **Memory Efficient**: Streams results to avoid loading entire files
//!
//! ## Search Result Format
//!
//! Each match includes:
//! - **File URI**: Valid URL pointing to the file
//! - **Line Number**: Zero-indexed line number of the match
//! - **Preview**: The matched text line
//!
//! Results are grouped by file, with each file containing multiple matches.
//

use std::{
	io,
	path::PathBuf,
	sync::{Arc, Mutex},
};

use CommonLibrary::{Error::CommonError::CommonError, Search::SearchProvider::SearchProvider};
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
			.Workspace
			.WorkspaceFolders
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
