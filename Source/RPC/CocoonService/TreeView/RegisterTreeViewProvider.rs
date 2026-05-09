#![allow(non_snake_case)]

//! Register a Cocoon-contributed tree-view provider in `ApplicationState`
//! and notify Sky via the coalesced `EnqueueTreeViewEmit` batcher.

use serde_json::json;

use tonic::{Response, Status};

use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;

use crate::{
	ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO,
	RPC::CocoonService::{CocoonServiceImpl, TreeView::EnqueueTreeViewEmit},
	Vine::Generated::{Empty, RegisterTreeViewProviderRequest},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:RegisterTreeViewProviderRequest,
) -> Result<Response<Empty>, Status> {

	dev_log!("cocoon", "[CocoonService] Registering tree view provider: {}", Request.view_id);

	let Handle = Request
		.view_id
		.as_bytes()
		.iter()
		.fold(0u32, |Acc, B| Acc.wrapping_mul(31).wrapping_add(*B as u32));

	let DTO = ProviderRegistrationDTO {

		Handle,

		ProviderType:ProviderType::TreeView,

		Selector:json!([{ "viewId": Request.view_id }]),

		SideCarIdentifier:"cocoon-main".to_string(),

		ExtensionIdentifier:json!(Request.extension_id),

		Options:Some(json!({ "viewId": Request.view_id })),
	};

	Service
		.environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.RegisterProvider(Handle, DTO);

	EnqueueTreeViewEmit::Fn(
		&Service.environment.ApplicationHandle,

		json!({ "viewId": Request.view_id, "extensionId": Request.extension_id }),
	);

	Ok(Response::new(Empty {}))
}
