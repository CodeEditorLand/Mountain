#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Search handlers - find in files, find files by glob.

use std::{path::PathBuf, sync::Arc};

use serde_json::{Value, json};

use crate::{dev_log, RunTime::ApplicationRunTime::ApplicationRunTime};

/// Search text across all workspace files (line-by-line grep, max 1000 results).
pub async fn handle_search_find_in_files(
	runtime:Arc<ApplicationRunTime>,
	args:Vec<Value>,
) -> Result<Value, String> {
	use globset::GlobBuilder;
	use tokio::fs;

	let Pattern = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("search:findInFiles requires pattern".to_string())?
		.to_string();
	let IsRegex = args.get(1).and_then(|V| V.as_bool()).unwrap_or(false);
	let IsCaseSensitive = args.get(2).and_then(|V| V.as_bool()).unwrap_or(false);
	let _IsWordMatch = args.get(3).and_then(|V| V.as_bool()).unwrap_or(false);
	let IncludeGlob = args.get(4).and_then(|V| V.as_str()).unwrap_or("**").to_string();
	let ExcludeGlob = args.get(5).and_then(|V| V.as_str()).unwrap_or("").to_string();
	let MaxResults = args.get(6).and_then(|V| V.as_u64()).unwrap_or(1000) as usize;

	let WorkspaceFolders = runtime.Environment.ApplicationState.Workspace.GetWorkspaceFolders();

	if WorkspaceFolders.is_empty() {
		return Ok(json!([]));
	}

	let RootPath = PathBuf::from(&WorkspaceFolders[0].URI.to_string().replace("file://", ""));

	// Build include matcher
	let IncludeMatcher = GlobBuilder::new(&IncludeGlob)
		.literal_separator(false)
		.build()
		.map(|G| G.compile_matcher())
		.ok();

	// Build exclude matcher
	let ExcludeMatcher = if !ExcludeGlob.is_empty() {
		GlobBuilder::new(&ExcludeGlob)
			.literal_separator(false)
			.build()
			.map(|G| G.compile_matcher())
			.ok()
	} else {
		None
	};

	let SearchText = Pattern.clone();
	let mut Matches = Vec::new();

	// Walk directory recursively
	let mut Stack = vec![RootPath.clone()];
	while let Some(Dir) = Stack.pop() {
		let mut Entries = match fs::read_dir(&Dir).await {
			Ok(E) => E,
			Err(_) => continue,
		};

		while let Ok(Some(Entry)) = Entries.next_entry().await {
			let Path = Entry.path();
			let RelPath = Path.strip_prefix(&RootPath).unwrap_or(&Path).to_string_lossy().to_string();

			// Skip hidden dirs
			if Path.file_name().map(|N| N.to_string_lossy().starts_with('.')).unwrap_or(false) {
				continue;
			}

			if Path.is_dir() {
				Stack.push(Path);
				continue;
			}

			// Check include/exclude globs
			if let Some(Ref) = &IncludeMatcher {
				if !Ref.is_match(&RelPath) {
					continue;
				}
			}
			if let Some(Ref) = &ExcludeMatcher {
				if Ref.is_match(&RelPath) {
					continue;
				}
			}

			// Read file and search line by line
			let Content = match fs::read_to_string(&Path).await {
				Ok(C) => C,
				Err(_) => continue,
			};

			for (LineIndex, Line) in Content.lines().enumerate() {
				let Hit = if IsRegex {
					// Simple contains fallback (no regex crate available here)
					Line.contains(&SearchText)
				} else if IsCaseSensitive {
					Line.contains(&SearchText)
				} else {
					Line.to_lowercase().contains(&SearchText.to_lowercase())
				};

				if Hit {
					let Uri = format!("file://{}", Path.to_string_lossy());
					Matches.push(json!({
						"uri": Uri,
						"lineNumber": LineIndex + 1,
						"preview": Line.trim(),
					}));

					if Matches.len() >= MaxResults {
						return Ok(json!(Matches));
					}
				}
			}
		}
	}

	Ok(json!(Matches))
}

/// Search file paths by glob pattern in workspace.
pub async fn handle_search_find_files(
	runtime:Arc<ApplicationRunTime>,
	args:Vec<Value>,
) -> Result<Value, String> {
	use globset::GlobBuilder;
	use tokio::fs;

	let Pattern = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("search:findFiles requires pattern".to_string())?
		.to_string();
	let MaxResults = args.get(1).and_then(|V| V.as_u64()).unwrap_or(500) as usize;

	let WorkspaceFolders = runtime.Environment.ApplicationState.Workspace.GetWorkspaceFolders();

	if WorkspaceFolders.is_empty() {
		return Ok(json!([]));
	}

	let RootPath = PathBuf::from(&WorkspaceFolders[0].URI.to_string().replace("file://", ""));

	let Matcher = GlobBuilder::new(&Pattern)
		.literal_separator(false)
		.build()
		.map(|G| G.compile_matcher())
		.map_err(|Error| format!("Invalid glob pattern: {}", Error))?;

	let mut Files = Vec::new();
	let mut Stack = vec![RootPath.clone()];

	while let Some(Dir) = Stack.pop() {
		let mut Entries = match fs::read_dir(&Dir).await {
			Ok(E) => E,
			Err(_) => continue,
		};

		while let Ok(Some(Entry)) = Entries.next_entry().await {
			let Path = Entry.path();

			if Path.file_name().map(|N| N.to_string_lossy().starts_with('.')).unwrap_or(false) {
				continue;
			}

			if Path.is_dir() {
				Stack.push(Path);
				continue;
			}

			let RelPath = Path.strip_prefix(&RootPath).unwrap_or(&Path).to_string_lossy().to_string();

			if Matcher.is_match(&RelPath) {
				Files.push(format!("file://{}", Path.to_string_lossy()));

				if Files.len() >= MaxResults {
					return Ok(json!(Files));
				}
			}
		}
	}

	Ok(json!(Files))
}
