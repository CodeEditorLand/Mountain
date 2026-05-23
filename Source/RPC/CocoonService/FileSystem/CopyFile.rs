
//! Copy a file, creating any missing target parents first.

use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{CopyFileRequest, Empty},
	dev_log,
};

pub async fn Fn(_Service:&CocoonServiceImpl, Request:CopyFileRequest) -> Result<Response<Empty>, Status> {
	let SourcePath = CocoonServiceImpl::UriToPath(Request.source.as_ref())
		.ok_or_else(|| Status::invalid_argument("copy_file: missing source URI"))?;

	let DestinationPath = CocoonServiceImpl::UriToPath(Request.target.as_ref())
		.ok_or_else(|| Status::invalid_argument("copy_file: missing target URI"))?;

	dev_log!("cocoon", "[CocoonService] copy_file: {:?} → {:?}", SourcePath, DestinationPath);

	if let Some(Parent) = DestinationPath.parent() {
		if !Parent.as_os_str().is_empty() {
			tokio::fs::create_dir_all(Parent)
				.await
				.map_err(|Error| Status::internal(format!("copy_file: create_dir_all failed: {}", Error)))?;
		}
	}

	tokio::fs::copy(&SourcePath, &DestinationPath)
		.await
		.map_err(|Error| Status::internal(format!("copy_file: {}: {}", SourcePath.display(), Error)))?;

	Ok(Response::new(Empty {}))
}
