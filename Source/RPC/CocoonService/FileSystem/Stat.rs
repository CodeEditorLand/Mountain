//! Inspect a path: type, size, mtime in ms-since-epoch.

use std::time::UNIX_EPOCH;

use tonic::{Response, Status};
use ::Vine::Generated::{StatRequest, StatResponse};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(_Service:&CocoonServiceImpl, Request:StatRequest) -> Result<Response<StatResponse>, Status> {
	let Path = CocoonServiceImpl::UriToPath(Request.uri.as_ref())
		.ok_or_else(|| Status::invalid_argument("stat: missing or empty URI"))?;

	dev_log!("cocoon", "[CocoonService] Stat: {:?}", Path);

	let Metadata = tokio::fs::metadata(&Path).await.map_err(|Error| {
		dev_log!("cocoon", "warn: [CocoonService] stat failed for {:?}: {}", Path, Error);

		Status::not_found(format!("stat: {}: {}", Path.display(), Error))
	})?;

	let MTime = Metadata
		.modified()
		.ok()
		.and_then(|T| T.duration_since(UNIX_EPOCH).ok())
		.map(|D| D.as_millis() as u64)
		.unwrap_or(0);

	Ok(Response::new(StatResponse {
		is_file:Metadata.is_file(),
		is_directory:Metadata.is_dir(),
		size:Metadata.len(),
		mtime:MTime,
	}))
}
