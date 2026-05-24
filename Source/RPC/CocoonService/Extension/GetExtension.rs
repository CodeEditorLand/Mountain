//! Look up a single scanned extension by id and project the manifest into
//! the gRPC `ExtensionInfo` shape.

use tonic::{Response, Status};
use CommonLibrary::ExtensionManagement::ExtensionManagementService::ExtensionManagementService;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{ExtensionInfo, GetExtensionRequest, GetExtensionResponse},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:GetExtensionRequest,
) -> Result<Response<GetExtensionResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] get_extension: {}", Request.ExtensionId);

	let Found = Service
		.environment
		.GetExtension(Request.ExtensionId.clone())
		.await
		.ok()
		.flatten();

	let Info = Found.map(|Value| {
		ExtensionInfo {
			id:Request.ExtensionId,
			display_name:Value.get("Name").and_then(|V| V.as_str()).unwrap_or("").to_string(),
			version:Value.get("Version").and_then(|V| V.as_str()).unwrap_or("").to_string(),
			is_active:true,
			extension_path:Value
				.Get("ExtensionLocation")
				.and_then(|V| V.as_str())
				.unwrap_or("")
				.to_string(),
		}
	});

	Ok(Response::new(GetExtensionResponse { extension:Info }))
}
