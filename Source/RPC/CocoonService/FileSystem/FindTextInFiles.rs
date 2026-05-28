//! Substring search across the workspace, capped at 1,000 matches. Skips
//! hidden directories plus `node_modules` and `target`. Runs the walk in
//! `tokio::task::spawn_blocking` so the event loop stays responsive.

use tonic::{Response, Status};
use ::Vine::Generated::{FindTextInFilesRequest, FindTextInFilesResponse, Position, Range, TextMatch, Uri};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:FindTextInFilesRequest,
) -> Result<Response<FindTextInFilesResponse>, Status> {
	if Request.pattern.is_empty() {
		return Ok(Response::new(FindTextInFilesResponse::default()));
	}

	dev_log!("cocoon", "[CocoonService] find_text_in_files: pattern='{}'", Request.pattern);

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

	let Pattern = Request.pattern.clone();

	let Matches = tokio::task::spawn_blocking(move || {
		let mut Results:Vec<TextMatch> = Vec::new();

		const MAX_MATCHES:usize = 1000;

		fn WalkAndSearch(Directory:&std::path::Path, Pattern:&str, Results:&mut Vec<TextMatch>) {
			if Results.len() >= MAX_MATCHES {
				return;
			}

			if let Ok(Entries) = std::fs::read_dir(Directory) {
				for Entry in Entries.flatten() {
					if Results.len() >= MAX_MATCHES {
						break;
					}

					let Path = Entry.path();

					if Path.is_dir() {
						let Name = Path.file_name().and_then(|N| N.to_str()).unwrap_or("");

						if Name.starts_with('.') || Name == "node_modules" || Name == "target" {
							continue;
						}

						WalkAndSearch(&Path, Pattern, Results);
					} else if Path.is_file() {
						if let Ok(Content) = std::fs::read_to_string(&Path) {
							for (LineIndex, Line) in Content.lines().enumerate() {
								if Results.len() >= MAX_MATCHES {
									break;
								}

								if let Some(ColumnIndex) = Line.find(Pattern) {
									Results.push(TextMatch {
										uri:Some(Uri { value:format!("file://{}", Path.display()) }),
										range:Some(Range {
											start:Some(Position {
												line:LineIndex as u32,
												character:ColumnIndex as u32,
											}),
											end:Some(Position {
												line:LineIndex as u32,
												character:(ColumnIndex + Pattern.len()) as u32,
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

	dev_log!(
		"cocoon",
		"[CocoonService] find_text_in_files: {} matches for '{}'",
		Matches.len(),
		Request.pattern
	);

	Ok(Response::new(FindTextInFilesResponse { matches:Matches }))
}
