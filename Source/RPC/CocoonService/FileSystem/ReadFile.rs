#![allow(non_snake_case)]

//! Read a file from disk and return its bytes (always tagged `utf-8` -
//! the encoding negotiation lives in Cocoon).

use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{ReadFileRequest, ReadFileResponse},
	dev_log,
};

pub async fn Fn(_Service:&CocoonServiceImpl, Request:ReadFileRequest) -> Result<Response<ReadFileResponse>, Status> {
	let Path = CocoonServiceImpl::UriToPath(Request.uri.as_ref())
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
