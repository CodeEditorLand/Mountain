//! Workspace text search delegated to the environment `SearchProvider`
//! (ripgrep-backed: parallel walk, gitignore-aware, literal patterns
//! escaped before compilation). Maps the provider's `FileMatch` JSON
//! into this RPC's `TextMatch` entries.

use serde_json::{Value, json};
use tonic::{Response, Status};
use CommonLibrary::Search::SearchProvider::SearchProvider;
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

	let QueryValue = json!({
		"pattern": Request.pattern,
		"isRegExp": false,
		"isCaseSensitive": false,
		"isWordMatch": false,
	});

	let OptionsValue = json!({
		"include": if Request.include.is_empty() { Value::Null } else { json!(Request.include) },
		"exclude": if Request.exclude.is_empty() { Value::Null } else { json!(Request.exclude) },
		"maxResults": 1000,
	});

	let Results = Service
		.environment
		.TextSearch(QueryValue, OptionsValue)
		.await
		.map_err(|Error| Status::internal(format!("find_text_in_files: {}", Error)))?;

	let mut Matches:Vec<TextMatch> = Vec::new();

	for FileMatch in Results.as_array().map(|A| A.as_slice()).unwrap_or_default() {
		let Resource = FileMatch.get("resource").and_then(|R| R.as_str()).unwrap_or("");

		if Resource.is_empty() {
			continue;
		}

		let FileMatches = FileMatch
			.get("matches")
			.and_then(|M| M.as_array())
			.map(|A| A.as_slice())
			.unwrap_or_default();

		for Match in FileMatches {
			let Preview = Match.get("preview").and_then(|P| P.as_str()).unwrap_or("");

			// `lineNumber` is 1-based from the provider; proto positions
			// are 0-based.
			let Line = Match
				.get("lineNumber")
				.and_then(|L| L.as_u64())
				.unwrap_or(1)
				.saturating_sub(1) as u32;

			let Columns = Match.get("columns").and_then(|C| C.as_array()).map(|A| A.as_slice()).unwrap_or_default();

			// Empty `columns` means match-position lookup failed in the
			// provider; highlight the whole preview line instead.
			let Ranges:Vec<(u32, u32)> = if Columns.is_empty() {
				vec![(0, Preview.chars().count() as u32)]
			} else {
				Columns
					.iter()
					.map(|C| {
						(
							C.get("start").and_then(|S| S.as_u64()).unwrap_or(0) as u32,
							C.get("end").and_then(|E| E.as_u64()).unwrap_or(0) as u32,
						)
					})
					.collect()
			};

			for (Start, End) in Ranges {
				Matches.push(TextMatch {
					uri:Some(Uri { value:Resource.to_string() }),
					range:Some(Range {
						start:Some(Position { line:Line, character:Start }),
						end:Some(Position { line:Line, character:End }),
					}),
					preview:Preview.to_string(),
				});
			}
		}
	}

	dev_log!(
		"cocoon",
		"[CocoonService] find_text_in_files: {} matches for '{}'",
		Matches.len(),
		Request.pattern
	);

	Ok(Response::new(FindTextInFilesResponse { matches:Matches }))
}
