#![allow(non_snake_case)]

//! Remove a file or recursively remove a directory.

use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{DeleteFileRequest, Empty},
	dev_log,
};

pub async fn Fn(_Service:&CocoonServiceImpl, Request:DeleteFileRequest) -> Result<Response<Empty>, Status> {

	let Path = CocoonServiceImpl::UriToPath(Request.uri.as_ref())
		.ok_or_else(|| Status::invalid_argument("delete_file: missing URI"))?;

	dev_log!("cocoon", "[CocoonService] delete_file: {:?}", Path);

	if Path.is_dir() {

		tokio::fs::remove_dir_all(&Path).await
	} else {

		tokio::fs::remove_file(&Path).await
	}
	.map_err(|Error| Status::internal(format!("delete_file: {}: {}", Path.display(), Error)))?;

	Ok(Response::new(Empty {}))
}
