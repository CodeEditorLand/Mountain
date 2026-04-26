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
use grep_matcher::Matcher;
use grep_regex::{RegexMatcher, RegexMatcherBuilder};
use grep_searcher::{Searcher, SearcherBuilder, Sink, SinkMatch};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::dev_log;

/// Mirrors VS Code's `ITextSearchQuery` shape (`vs/workbench/services/
/// search/common/search.ts`). The workbench's Search view serialises
/// the user's input into this struct and the ProxyChannel sends it as
/// slot 0 of the `search:textSearch` call.
///
/// - `pattern`: the user's typed query
/// - `isRegExp` (default `false`): when `false`, the pattern is
///   `regex::escape`'d before compilation so a literal search for `obj.method(`
///   doesn't blow up the regex parser.
/// - `isCaseSensitive` (default `false`): controls the regex's case-insensitive
///   flag.
/// - `isWordMatch` (default `false`): wraps the pattern in `\b…\b` via
///   `RegexMatcherBuilder::word(true)`.
/// - `isMultiline` (default `false`): toggles `.` matching `\n`.
#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
struct TextSearchQuery {
	pattern:String,

	#[serde(default)]
	is_case_sensitive:Option<bool>,

	#[serde(default)]
	is_word_match:Option<bool>,

	#[serde(default)]
	is_reg_exp:Option<bool>,

	#[serde(default)]
	is_multiline:Option<bool>,
}

/// Per-match column range within the preview line.
///
/// `start` and `end` are 0-based UTF-8 character offsets, NOT byte
/// offsets - VS Code's renderer measures columns in code units, so
/// pre-converting bytes→chars here keeps the workbench from
/// mis-highlighting multi-byte UTF-8 lines (the search panel underlines
/// the wrong substring otherwise).
///
/// VS Code's `ISearchRange` is 1-based for line numbers but 0-based
/// for columns; the SkyBridge consumer adds the +1 line offset there.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct ColumnRange {
	start:u64,

	end:u64,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct TextMatch {
	preview:String,

	/// 1-based line number (grep-searcher emits 1-based when
	/// `line_number(true)` is configured on the SearcherBuilder).
	line_number:u64,

	/// Per-line ranges where the matcher actually matched. A single
	/// line can contain multiple matches (e.g. `test test test`); each
	/// gets its own range. Empty when match-position lookup failed -
	/// in that case the renderer falls back to highlighting the whole
	/// line.
	columns:Vec<ColumnRange>,
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

	/// Cloned per-thread so the sink can re-run the matcher against the
	/// raw line bytes to recover column ranges. `SinkMatch::bytes()`
	/// gives us the matched line but not where in the line the matcher
	/// hit; calling `Matcher::find_at(...)` ourselves is the documented
	/// pattern for recovering that information.
	matcher:RegexMatcher,
}

impl Sink for PerFileSink {
	type Error = io::Error;

	fn matched(&mut self, _Searcher:&Searcher, Mat:&SinkMatch<'_>) -> Result<bool, Self::Error> {
		let RawLine = Mat.bytes();
		// Trim trailing newline so the preview text the renderer shows
		// doesn't carry a stray empty line break.
		let TrimmedLen = if RawLine.ends_with(b"\r\n") {
			RawLine.len().saturating_sub(2)
		} else if RawLine.last() == Some(&b'\n') {
			RawLine.len().saturating_sub(1)
		} else {
			RawLine.len()
		};
		let LineBytes = &RawLine[..TrimmedLen];
		// Cap preview length at 512 chars - super-long minified lines
		// would otherwise force the renderer to layout massive rows
		// AND make the byte→char map below grow proportionally.
		const PREVIEW_BYTE_CAP:usize = 512;
		let CapBytes = LineBytes.len().min(PREVIEW_BYTE_CAP);
		// Round down to the nearest UTF-8 boundary so `from_utf8_lossy`
		// doesn't replace half a multibyte char with U+FFFD.
		let SafeCap = (0..=CapBytes)
			.rev()
			.find(|&I| I == 0 || I == LineBytes.len() || (LineBytes[I] & 0xC0) != 0x80)
			.unwrap_or(0);
		let Preview = String::from_utf8_lossy(&LineBytes[..SafeCap]).to_string();

		// `line_number(true)` was set on the SearcherBuilder so this
		// returns Some(n) (1-based). Default to 1 if we somehow lose
		// it - rendering "line 0" looked wrong even when the rest of
		// the data was correct.
		let LineNumber = Mat.line_number().unwrap_or(1);

		// Build a byte→char map ONCE per line so every column lookup
		// is O(log n) (binary search) instead of O(n) (the previous
		// `char_indices().position()` per call). On lines with many
		// matches this collapses the per-line work from quadratic to
		// linear, which is the difference between a 6 s search and a
		// minutes-long hang on workspaces that contain match-dense
		// minified bundles.
		let mut CharBoundaries:Vec<usize> = Vec::with_capacity(Preview.len() / 2 + 1);
		for (B, _) in Preview.char_indices() {
			CharBoundaries.push(B);
		}
		CharBoundaries.push(Preview.len()); // Sentinel for end-of-string.
		let ByteToChar = |Byte:usize| -> u64 {
			match CharBoundaries.binary_search(&Byte) {
				Ok(Index) => Index as u64,
				Err(Index) => Index as u64,
			}
		};

		// Walk the line bytes and collect every sub-line range the
		// matcher hits. Multiple matches per line are common
		// (e.g. searching for `test` in `test test`); each becomes its
		// own ColumnRange so the renderer underlines them all. Cap at
		// `MAX_COLUMNS_PER_LINE` to bound work on pathological lines
		// where a regex matches every character (e.g. `.` or `\w`
		// against a long minified line).
		const MAX_COLUMNS_PER_LINE:usize = 100;
		let mut Columns:Vec<ColumnRange> = Vec::new();
		let mut StartByte = 0usize;
		// Search within the truncated preview so columns line up with
		// the preview text the renderer will display.
		let SearchBytes = &LineBytes[..SafeCap];
		while StartByte <= SearchBytes.len() && Columns.len() < MAX_COLUMNS_PER_LINE {
			match self.matcher.find_at(SearchBytes, StartByte) {
				Ok(Some(M)) => {
					if M.start() >= SearchBytes.len() {
						break;
					}
					Columns.push(ColumnRange { start:ByteToChar(M.start()), end:ByteToChar(M.end()) });
					// `M.end() == M.start()` happens for zero-width
					// matches (e.g. `\b`); advance by one byte to
					// avoid an infinite loop.
					StartByte = if M.end() == M.start() { M.end() + 1 } else { M.end() };
				},
				_ => break,
			}
		}

		// Since this sink is per-file, we know `self.path` is correct.
		let FileURI = url::Url::from_file_path(&self.path)
			.map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Could not convert path to URL"))?
			.to_string();

		let NewMatch = TextMatch { preview:Preview, line_number:LineNumber, columns:Columns };

		// Mutex acquired AFTER the column-range scan so contention
		// doesn't serialise the per-line regex work across the
		// `WalkBuilder::build_parallel()` workers.
		let mut ResultsGuard = self
			.results
			.lock()
			.map_err(|Error| io::Error::new(io::ErrorKind::Other, Error.to_string()))?;

		// Find the entry for our file, or create it if it's the first match.
		if let Some(FileMatch) = ResultsGuard.iter_mut().find(|fm| fm.resource == FileURI) {
			FileMatch.matches.push(NewMatch);
		} else {
			ResultsGuard.push(FileMatch { resource:FileURI, matches:vec![NewMatch] });
		}

		// Continue searching
		Ok(true)
	}
}

#[async_trait]
impl SearchProvider for MountainEnvironment {
	async fn TextSearch(&self, QueryValue:Value, _OptionsValue:Value) -> Result<Value, CommonError> {
		let Query:TextSearchQuery = serde_json::from_value(QueryValue)?;

		dev_log!("search", "[SearchProvider] Performing text search for: {:?}", Query);

		let mut Builder = RegexMatcherBuilder::new();

		Builder
			.case_insensitive(!Query.is_case_sensitive.unwrap_or(false))
			.word(Query.is_word_match.unwrap_or(false))
			.multi_line(Query.is_multiline.unwrap_or(false));

		// When `isRegExp` is false/missing (the default for the Search
		// view's plain-text mode), escape the pattern so literal
		// searches for strings containing regex metacharacters
		// (`.`, `(`, `[`, `*`, `?`, etc.) don't crash the compiler
		// or silently match the wrong thing.
		let CompiledPattern = if Query.is_reg_exp.unwrap_or(false) {
			Query.pattern.clone()
		} else {
			regex::escape(&Query.pattern)
		};

		let Matcher = Builder.build(&CompiledPattern).map_err(|Error| {
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
			dev_log!("search", "warn: [SearchProvider] No workspace folders to search in.");

			return Ok(json!([]));
		}

		for Folder in Folders {
			if let Ok(FolderPath) = Folder.URI.to_file_path() {
				// Use a parallel walker for better performance.
				let Walker = WalkBuilder::new(FolderPath).build_parallel();

				// The `search_parallel` method is not available on `Searcher`. We must process
				// entries from the walker and call `search_path` individually.
				Walker.run(|| {
					// `line_number(true)` is mandatory - without it,
					// `SinkMatch::line_number()` returns None and every
					// match lands at line 0, which the renderer treats
					// as "no line info" and collapses into an
					// uncategorised count-of-zero. The default
					// `Searcher::new()` constructor disables line
					// numbers for performance.
					let mut Searcher = SearcherBuilder::new().line_number(true).build();

					let Matcher = Matcher.clone();

					let AllMatches = AllMatches.clone();

					Box::new(move |EntryResult| {
						if let Ok(Entry) = EntryResult {
							if Entry.file_type().map_or(false, |ft| ft.is_file()) {
								// For each file, create a new sink that knows its path.
								let Sink = PerFileSink {
									path:Entry.path().to_path_buf(),
									results:AllMatches.clone(),
									matcher:Matcher.clone(),
								};

								if let Err(Error) = Searcher.search_path(&Matcher, Entry.path(), Sink) {
									dev_log!(
										"search",
										"warn: [SearchProvider] Error searching path {}: {}",
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

		let TotalLineMatches:usize = FinalMatches.iter().map(|F| F.matches.len()).sum();
		dev_log!(
			"search",
			"[SearchProvider] returned {} files / {} line-matches for pattern={:?}",
			FinalMatches.len(),
			TotalLineMatches,
			Query.pattern
		);

		Ok(json!(FinalMatches))
	}
}
