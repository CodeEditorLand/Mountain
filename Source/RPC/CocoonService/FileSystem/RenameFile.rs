//! Rename a file or directory, creating any missing target parents first.
use tonic::{Response, Status};
use ::Vine::Generated::{Empty, RenameFileRequest};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(_Service:&CocoonServiceImpl, Request:RenameFileRequest) -> Result<Response<Empty>, Status> {
	let OldPath = CocoonServiceImpl::UriToPath(Request.source.as_ref())
		.ok_or_else(|| Status::invalid_argument("rename_file: missing source URI"))?;

	let NewPath = CocoonServiceImpl::UriToPath(Request.target.as_ref())
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
