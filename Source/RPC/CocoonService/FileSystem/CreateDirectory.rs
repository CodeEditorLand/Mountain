
//! Create a directory (and any missing parents).

use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{CreateDirectoryRequest, Empty},
	dev_log,
};

pub async fn Fn(_Service:&CocoonServiceImpl, Request:CreateDirectoryRequest) -> Result<Response<Empty>, Status> {
	let Path = CocoonServiceImpl::UriToPath(Request.uri.as_ref())
		.ok_or_else(|| Status::invalid_argument("create_directory: missing URI"))?;

	dev_log!("cocoon", "[CocoonService] create_directory: {:?}", Path);

	tokio::fs::create_dir_all(&Path)
		.await
		.map_err(|Error| Status::internal(format!("create_directory: {}: {}", Path.display(), Error)))?;

	Ok(Response::new(Empty {}))
}
