//! Register a Cocoon-contributed task provider in `ApplicationState`. The
//! gRPC proto carries no handle, so we hash the task `type` string for
//! the registration handle.

use serde_json::json;
use tonic::{Response, Status};
use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;

use crate::{
	ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO,
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, RegisterTaskProviderRequest},
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:RegisterTaskProviderRequest) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering Task Provider: type={}", Request.r#type);

	let Handle = Request
		.r#type
		.as_bytes()
		.iter()
		.fold(0u32, |Acc, B| Acc.wrapping_mul(31).wrapping_add(*B as u32));

	let DTO = ProviderRegistrationDTO {
		Handle,

		ProviderType:ProviderType::Task,

		Selector:json!([{ "language": "*" }]),

		SideCarIdentifier:"cocoon-main".to_string(),

		ExtensionIdentifier:json!(Request.extension_id),

		Options:None,
	};

	Service
		.environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.RegisterProvider(Handle, DTO);

	Ok(Response::new(Empty {}))
}
