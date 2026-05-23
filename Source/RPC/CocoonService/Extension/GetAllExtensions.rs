//! Return every scanned extension projected into the gRPC `ExtensionInfo`
//! shape.

use tonic::{Response, Status};
use CommonLibrary::ExtensionManagement::ExtensionManagementService::ExtensionManagementService;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, ExtensionInfo, GetAllExtensionsResponse},
};

pub async fn Fn(Service:&CocoonServiceImpl, _Request:Empty) -> Result<Response<GetAllExtensionsResponse>, Status> {
	let Extensions = Service.environment.GetExtensions().await.unwrap_or_default();

	let List = Extensions
		.iter()
		.map(|Value| {
			ExtensionInfo {
				id:Value.get("Identifier").and_then(|V| V.as_str()).unwrap_or("").to_string(),
				display_name:Value.get("Name").and_then(|V| V.as_str()).unwrap_or("").to_string(),
				version:Value.get("Version").and_then(|V| V.as_str()).unwrap_or("").to_string(),
				is_active:true,
				extension_path:Value
					.get("ExtensionLocation")
					.and_then(|V| V.as_str())
					.unwrap_or("")
					.to_string(),
			}
		})
		.collect();

	Ok(Response::new(GetAllExtensionsResponse { extensions:List }))
}
