#![allow(non_snake_case)]
//! File System domain handlers for CocoonService.
//!
//! Typed gRPC RPCs: read_file, write_file, stat, readdir, watch_file,
//! find_files, find_text_in_files, delete_file, rename_file, copy_file,
//! create_directory.

use std::time::UNIX_EPOCH;

use serde_json::json;
use tonic::{Response, Status};

use super::CocoonServiceImpl;
use crate::dev_log;
use crate::Vine::Generated::{
	CopyFileRequest, CreateDirectoryRequest, DeleteFileRequest, Empty,
	FindFilesRequest, FindFilesResponse, FindTextInFilesRequest,
	FindTextInFilesResponse, Position, Range, ReadFileRequest,
	ReadFileResponse, ReaddirRequest, ReaddirResponse, RenameFileRequest,
	StatRequest, StatResponse, TextMatch, Uri, WatchFileRequest,
	WriteFileRequest,
};

pub async fn ReadFile(
	Service:&CocoonServiceImpl,
	req:ReadFileRequest,
) -> Result<Response<ReadFileResponse>, Status> {
	let Path = CocoonServiceImpl::UriToPath(req.uri.as_ref())
		.ok_or_else(|| Status::invalid_argument("read_file: missing or empty URI"))?;

	dev_log!("cocoon", "[CocoonService] Reading file: {:?}", Path);

	let Content = tokio::fs::read(&Path).await.map_err(|Error| {
		dev_log!("cocoon", "warn: [CocoonService] read_file failed for {:?}: {}", Path, Error);
		Status::not_found(format!("read_file: {}: {}", Path.display(), Error))
	})?;

	Ok(Response::new(ReadFileResponse {
		content:Content,
		encoding:"utf-8".to_string(),
	}))
}

pub async fn WriteFile(
	Service:&CocoonServiceImpl,
	req:WriteFileRequest,
) -> Result<Response<Empty>, Status> {
	let Path = CocoonServiceImpl::UriToPath(req.uri.as_ref())
		.ok_or_else(|| Status::invalid_argument("write_file: missing or empty URI"))?;

	dev_log!("cocoon", "[CocoonService] Writing file: {:?} ({} bytes)", Path, req.content.len());

	// Ensure parent directory exists
	if let Some(Parent) = Path.parent() {
		if !Parent.as_os_str().is_empty() {
			tokio::fs::create_dir_all(Parent)
				.await
				.map_err(|Error| Status::internal(format!("write_file: create_dir_all {:?}: {}", Parent, Error)))?;
		}
	}

	tokio::fs::write(&Path, &req.content).await.map_err(|Error| {
		dev_log!("cocoon", "warn: [CocoonService] write_file failed for {:?}: {}", Path, Error);
		Status::internal(format!("write_file: {}: {}", Path.display(), Error))
	})?;

	Ok(Response::new(Empty {}))
}

pub async fn Stat(
	Service:&CocoonServiceImpl,
	req:StatRequest,
) -> Result<Response<StatResponse>, Status> {
	let Path =
		CocoonServiceImpl::UriToPath(req.uri.as_ref()).ok_or_else(|| Status::invalid_argument("stat: missing or empty URI"))?;

	dev_log!("cocoon", "[CocoonService] Stat: {:?}", Path);

	let Metadata = tokio::fs::metadata(&Path).await.map_err(|Error| {
		dev_log!("cocoon", "warn: [CocoonService] stat failed for {:?}: {}", Path, Error);
		Status::not_found(format!("stat: {}: {}", Path.display(), Error))
	})?;

	let Mtime = Metadata
		.modified()
		.ok()
		.and_then(|T| T.duration_since(UNIX_EPOCH).ok())
		.map(|D| D.as_millis() as u64)
		.unwrap_or(0);

	Ok(Response::new(StatResponse {
		is_file:Metadata.is_file(),
		is_directory:Metadata.is_dir(),
		size:Metadata.len(),
		mtime:Mtime,
	}))
}

pub async fn Readdir(
	Service:&CocoonServiceImpl,
	req:ReaddirRequest,
) -> Result<Response<ReaddirResponse>, Status> {
	let Path = CocoonServiceImpl::UriToPath(req.uri.as_ref())
		.ok_or_else(|| Status::invalid_argument("readdir: missing or empty URI"))?;

	dev_log!("cocoon", "[CocoonService] Readdir: {:?}", Path);

	let mut ReadDir = tokio::fs::read_dir(&Path).await.map_err(|Error| {
		dev_log!("cocoon", "warn: [CocoonService] readdir failed for {:?}: {}", Path, Error);
		Status::not_found(format!("readdir: {}: {}", Path.display(), Error))
	})?;

	let mut Entries = Vec::new();
	while let Ok(Some(Entry)) = ReadDir.next_entry().await {
		if let Some(Name) = Entry.file_name().to_str() {
			Entries.push(Name.to_string());
		}
	}

	Ok(Response::new(ReaddirResponse { entries:Entries }))
}

pub async fn WatchFile(
	Service:&CocoonServiceImpl,
	req:WatchFileRequest,
) -> Result<Response<Empty>, Status> {
	let Uri = req.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");
	dev_log!("cocoon", "[CocoonService] watch_file registered (polling not yet active): {}", Uri);
	// TODO(P1): Wire notify crate watcher; store WatcherHandle in
	// ApplicationState.Feature.Watchers keyed by URI for cancellation on
	// cancel_operation.
	Ok(Response::new(Empty {}))
}

pub async fn FindFiles(
	Service:&CocoonServiceImpl,
	req:FindFilesRequest,
) -> Result<Response<FindFilesResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Finding files with pattern: {}", req.pattern);

	use globset::Glob;

	// Build glob matcher
	let GlobMatcher = Glob::new(&req.pattern)
		.map_err(|Error| {
			Status::invalid_argument(format!("find_files: invalid pattern '{}': {}", req.pattern, Error))
		})?
		.compile_matcher();

	// Collect workspace root folders from ApplicationState
	let Roots:Vec<std::path::PathBuf> = {
		match Service.environment.ApplicationState.Workspace.WorkspaceFolders.lock() {
			Ok(Guard) => Guard.iter().map(|F| std::path::PathBuf::from(F.URI.path())).collect(),
			Err(_) => Vec::new(),
		}
	};

	let SearchRoots = if Roots.is_empty() {
		vec![std::env::current_dir().unwrap_or_default()]
	} else {
		Roots
	};

	// Walk each root and collect matching paths
	let mut Uris = Vec::new();

	fn WalkAndCollect(
		Directory:&std::path::Path,
		Root:&std::path::Path,
		Matcher:&globset::GlobMatcher,
		Results:&mut Vec<String>,
	) {
		if let Ok(Entries) = std::fs::read_dir(Directory) {
			for Entry in Entries.flatten() {
				let EntryPath = Entry.path();
				if EntryPath.is_dir() {
					WalkAndCollect(&EntryPath, Root, Matcher, Results);
				} else if let Ok(Relative) = EntryPath.strip_prefix(Root) {
					if Matcher.is_match(Relative) {
						Results.push(format!("file://{}", EntryPath.display()));
					}
				}
			}
		}
	}

	for Root in &SearchRoots {
		WalkAndCollect(Root, Root, &GlobMatcher, &mut Uris);
	}

	dev_log!("cocoon",
		"[CocoonService] find_files: {} results for pattern '{}'",
		Uris.len(),
		req.pattern
	);
	Ok(Response::new(FindFilesResponse { uris:Uris }))
}

pub async fn FindTextInFiles(
	Service:&CocoonServiceImpl,
	req:FindTextInFilesRequest,
) -> Result<Response<FindTextInFilesResponse>, Status> {
	if req.pattern.is_empty() {
		return Ok(Response::new(FindTextInFilesResponse::default()));
	}
	dev_log!("cocoon", "[CocoonService] find_text_in_files: pattern='{}'", req.pattern);

	let Roots:Vec<std::path::PathBuf> = {
		match Service.environment.ApplicationState.Workspace.WorkspaceFolders.lock() {
			Ok(Guard) => Guard.iter().map(|F| std::path::PathBuf::from(F.URI.path())).collect(),
			Err(_) => Vec::new(),
		}
	};
	let SearchRoots = if Roots.is_empty() {
		vec![std::env::current_dir().unwrap_or_default()]
	} else {
		Roots
	};

	let Pattern = req.pattern.clone();
	let Matches = tokio::task::spawn_blocking(move || {
		let mut Results:Vec<TextMatch> = Vec::new();
		const MAX_MATCHES:usize = 1000;

		fn WalkAndSearch(Dir:&std::path::Path, Pattern:&str, Results:&mut Vec<TextMatch>) {
			if Results.len() >= 1000 {
				return;
			}
			if let Ok(Entries) = std::fs::read_dir(Dir) {
				for Entry in Entries.flatten() {
					if Results.len() >= MAX_MATCHES {
						break;
					}
					let Path = Entry.path();
					if Path.is_dir() {
						// Skip hidden dirs and common noise dirs
						let DirName = Path.file_name().and_then(|N| N.to_str()).unwrap_or("");
						if DirName.starts_with('.') || DirName == "node_modules" || DirName == "target" {
							continue;
						}
						WalkAndSearch(&Path, Pattern, Results);
					} else if Path.is_file() {
						if let Ok(Content) = std::fs::read_to_string(&Path) {
							for (LineIdx, Line) in Content.lines().enumerate() {
								if Results.len() >= MAX_MATCHES {
									break;
								}
								if let Some(ColIdx) = Line.find(Pattern) {
									Results.push(TextMatch {
										uri:Some(Uri { value:format!("file://{}", Path.display()) }),
										range:Some(Range {
											start:Some(Position { line:LineIdx as u32, character:ColIdx as u32 }),
											end:Some(Position {
												line:LineIdx as u32,
												character:(ColIdx + Pattern.len()) as u32,
											}),
										}),
										preview:Line.to_string(),
									});
								}
							}
						}
					}
				}
			}
		}

		for Root in &SearchRoots {
			WalkAndSearch(Root, &Pattern, &mut Results);
			if Results.len() >= MAX_MATCHES {
				break;
			}
		}
		Results
	})
	.await
	.unwrap_or_default();

	dev_log!("cocoon",
		"[CocoonService] find_text_in_files: {} matches for '{}'",
		Matches.len(),
		req.pattern
	);
	Ok(Response::new(FindTextInFilesResponse { matches:Matches }))
}

pub async fn DeleteFile(
	Service:&CocoonServiceImpl,
	req:DeleteFileRequest,
) -> Result<Response<Empty>, Status> {
	let Path =
		CocoonServiceImpl::UriToPath(req.uri.as_ref()).ok_or_else(|| Status::invalid_argument("delete_file: missing URI"))?;

	dev_log!("cocoon", "[CocoonService] delete_file: {:?}", Path);

	if Path.is_dir() {
		tokio::fs::remove_dir_all(&Path).await
	} else {
		tokio::fs::remove_file(&Path).await
	}
	.map_err(|Error| Status::internal(format!("delete_file: {}: {}", Path.display(), Error)))?;

	Ok(Response::new(Empty {}))
}

pub async fn RenameFile(
	Service:&CocoonServiceImpl,
	req:RenameFileRequest,
) -> Result<Response<Empty>, Status> {
	let OldPath = CocoonServiceImpl::UriToPath(req.source.as_ref())
		.ok_or_else(|| Status::invalid_argument("rename_file: missing source URI"))?;
	let NewPath = CocoonServiceImpl::UriToPath(req.target.as_ref())
		.ok_or_else(|| Status::invalid_argument("rename_file: missing target URI"))?;

	dev_log!("cocoon", "[CocoonService] rename_file: {:?} → {:?}", OldPath, NewPath);

	if let Some(Parent) = NewPath.parent() {
		if !Parent.as_os_str().is_empty() {
			tokio::fs::create_dir_all(Parent)
				.await
				.map_err(|Error| Status::internal(format!("rename_file: create_dir_all failed: {}", Error)))?;
		}
	}

	tokio::fs::rename(&OldPath, &NewPath)
		.await
		.map_err(|Error| Status::internal(format!("rename_file: {}: {}", OldPath.display(), Error)))?;

	Ok(Response::new(Empty {}))
}

pub async fn CopyFile(
	Service:&CocoonServiceImpl,
	req:CopyFileRequest,
) -> Result<Response<Empty>, Status> {
	let SrcPath = CocoonServiceImpl::UriToPath(req.source.as_ref())
		.ok_or_else(|| Status::invalid_argument("copy_file: missing source URI"))?;
	let DstPath = CocoonServiceImpl::UriToPath(req.target.as_ref())
		.ok_or_else(|| Status::invalid_argument("copy_file: missing target URI"))?;

	dev_log!("cocoon", "[CocoonService] copy_file: {:?} → {:?}", SrcPath, DstPath);

	if let Some(Parent) = DstPath.parent() {
		if !Parent.as_os_str().is_empty() {
			tokio::fs::create_dir_all(Parent)
				.await
				.map_err(|Error| Status::internal(format!("copy_file: create_dir_all failed: {}", Error)))?;
		}
	}

	tokio::fs::copy(&SrcPath, &DstPath)
		.await
		.map_err(|Error| Status::internal(format!("copy_file: {}: {}", SrcPath.display(), Error)))?;

	Ok(Response::new(Empty {}))
}

pub async fn CreateDirectory(
	Service:&CocoonServiceImpl,
	req:CreateDirectoryRequest,
) -> Result<Response<Empty>, Status> {
	let Path = CocoonServiceImpl::UriToPath(req.uri.as_ref())
		.ok_or_else(|| Status::invalid_argument("create_directory: missing URI"))?;

	dev_log!("cocoon", "[CocoonService] create_directory: {:?}", Path);

	tokio::fs::create_dir_all(&Path)
		.await
		.map_err(|Error| Status::internal(format!("create_directory: {}: {}", Path.display(), Error)))?;

	Ok(Response::new(Empty {}))
}
