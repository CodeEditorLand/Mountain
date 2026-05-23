
//! Write bytes to disk, creating any missing parent directories.

use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, WriteFileRequest},
	dev_log,
};

pub async fn Fn(_Service:&CocoonServiceImpl, Request:WriteFileRequest) -> Result<Response<Empty>, Status> {
	let Path = CocoonServiceImpl::UriToPath(Request.uri.as_ref())
		.ok_or_else(|| Status::invalid_argument("write_file: missing or empty URI"))?;

	dev_log!(
		"cocoon",
		"[CocoonService] Writing file: {:?} ({} bytes)",
		Path,
		Request.content.len()
	);

	if let Some(Parent) = Path.parent() {
		if !Parent.as_os_str().is_empty() {
			tokio::fs::create_dir_all(Parent)
				.await
				.map_err(|Error| Status::internal(format!("write_file: create_dir_all {:?}: {}", Parent, Error)))?;
		}
	}

	tokio::fs::write(&Path, &Request.content).await.map_err(|Error| {
		dev_log!("cocoon", "warn: [CocoonService] write_file failed for {:?}: {}", Path, Error);
		Status::internal(format!("write_file: {}: {}", Path.display(), Error))
	})?;

	Ok(Response::new(Empty {}))
}
