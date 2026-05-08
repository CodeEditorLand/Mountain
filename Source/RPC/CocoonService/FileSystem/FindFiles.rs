#![allow(non_snake_case)]

//! Walk every workspace root collecting paths that match `pattern`
//! (globset). Falls back to cwd when no roots are open.

use globset::Glob;
use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{FindFilesRequest, FindFilesResponse},
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:FindFilesRequest) -> Result<Response<FindFilesResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Finding files with pattern: {}", Request.pattern);

	let Matcher = Glob::new(&Request.pattern)
		.map_err(|Error| {
			Status::invalid_argument(format!("find_files: invalid pattern '{}': {}", Request.pattern, Error))
		})?
		.compile_matcher();

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

	let mut URIs = Vec::new();

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
		WalkAndCollect(Root, Root, &Matcher, &mut URIs);
	}

	dev_log!(
		"cocoon",
		"[CocoonService] find_files: {} results for pattern '{}'",
		URIs.len(),
		Request.pattern
	);

	Ok(Response::new(FindFilesResponse { uris:URIs }))
}
