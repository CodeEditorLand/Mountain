
//! Enumerate the entries of a directory by name.

use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{ReaddirRequest, ReaddirResponse},
	dev_log,
};

pub async fn Fn(_Service:&CocoonServiceImpl, Request:ReaddirRequest) -> Result<Response<ReaddirResponse>, Status> {
	let Path = CocoonServiceImpl::UriToPath(Request.uri.as_ref())
		.ok_or_else(|| Status::invalid_argument("readdir: missing or empty URI"))?;

	dev_log!("cocoon", "[CocoonService] Readdir: {:?}", Path);

	let mut ReadDir = tokio::fs::read_dir(&Path).await.map_err(|Error| {
		dev_log!("cocoon", "warn: [CocoonService] readdir failed for {:?}: {}", Path, Error);
		Status::not_found(format!("readdir: {}: {}", Path.display(), Error))
	})?;

	let mut Entries = Vec::new();

	while let Ok(Some(Entry)) = ReadDir.next_entry().await {
		if let Some(Name) = Entry.file_name().to_str() {
			Entries.push(Name.to_string());
		}
	}

	Ok(Response::new(ReaddirResponse { entries:Entries }))
}
