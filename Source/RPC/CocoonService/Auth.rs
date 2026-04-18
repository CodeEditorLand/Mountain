#![allow(non_snake_case)]
//! Authentication domain handlers for CocoonService.
//!
//! Typed gRPC RPCs: get_authentication_session,
//! register_authentication_provider.

use serde_json::json;
use tonic::{Response, Status};
use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;

use super::CocoonServiceImpl;
use crate::{
	ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO,
	Vine::Generated::{
		Empty,
		GetAuthenticationSessionRequest,
		GetAuthenticationSessionResponse,
		RegisterAuthenticationProviderRequest,
	},
	dev_log,
};

pub async fn GetAuthenticationSession(
	Service:&CocoonServiceImpl,
	req:GetAuthenticationSessionRequest,
) -> Result<Response<GetAuthenticationSessionResponse>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] get_authentication_session: provider={}",
		req.provider_id
	);

	// Return empty session — auth providers register themselves via
	// register_authentication_provider and get stored in ApplicationState.
	// The full OAuth flow requires Mountain to open a browser window.
	Ok(Response::new(GetAuthenticationSessionResponse::default()))
}

pub async fn RegisterAuthenticationProvider(
	Service:&CocoonServiceImpl,
	req:RegisterAuthenticationProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering Authentication Provider: id={}", req.id);

	let Handle = req
		.id
		.as_bytes()
		.iter()
		.fold(0u32, |Acc, B| Acc.wrapping_mul(31).wrapping_add(*B as u32));
	let dto = ProviderRegistrationDTO {
		Handle,
		ProviderType:ProviderType::Authentication,
		Selector:json!([{ "provider": req.id }]),
		SideCarIdentifier:"cocoon-main".to_string(),
		ExtensionIdentifier:json!(req.extension_id),
		Options:Some(json!({ "label": req.label, "supportsMultipleAccounts": req.supports_multiple_accounts })),
	};
	Service
		.environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.RegisterProvider(Handle, dto);

	Ok(Response::new(Empty {}))
}
