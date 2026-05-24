//! Register an authentication provider in `ApplicationState`. Cocoon-side
//! providers (GitHub, Microsoft, etc.) call this on activation; later
//! `GetAuthenticationSession` calls look up the registered handle.

use serde_json::json;
use tonic::{Response, Status};
use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;

use crate::{
	ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO,
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, RegisterAuthenticationProviderRequest},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:RegisterAuthenticationProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Registering Authentication Provider: id={}",
		Request.id
	);

	let Handle = Request
		.id
		.as_bytes()
		.iter()
		.fold(0u32, |Acc, B| Acc.wrapping_mul(31).wrapping_add(*B as u32));

	let DTO = ProviderRegistrationDTO {
		Handle,

		ProviderType:ProviderType::Authentication,

		Selector:json!([{ "provider": Request.id }]),

		SideCarIdentifier:"cocoon-main".to_string(),

		ExtensionIdentifier:json!(Request.ExtensionId),

		Options:Some(json!({
			"label": Request.label,
			"supportsMultipleAccounts": Request.supports_multiple_accounts,
		})),
	};

	Service
		.environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.RegisterProvider(Handle, DTO);

	Ok(Response::new(Empty {}))
}
