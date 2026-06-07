//! Register a Cocoon-contributed debug adapter in `ApplicationState` and
//! notify Sky so the debug-launcher UI can light up.

use serde_json::json;

use tauri::Emitter;

use tonic::{Response, Status};

use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;

use ::Vine::Generated::{Empty, RegisterDebugAdapterRequest};

use crate::{
	ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO,
	RPC::CocoonService::CocoonServiceImpl,
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:RegisterDebugAdapterRequest) -> Result<Response<Empty>, Status> {

	dev_log!("cocoon", "[CocoonService] Registering debug adapter: {}", Request.debug_type);

	let Handle = Request
		.debug_type
		.as_bytes()
		.iter()
		.fold(0u32, |Acc, B| Acc.wrapping_mul(31).wrapping_add(*B as u32));

	let DTO = ProviderRegistrationDTO {
		Handle,

		ProviderType:ProviderType::DebugAdapter,

		Selector:json!([{ "debugType": Request.debug_type }]),

		SideCarIdentifier:"cocoon-main".to_string(),

		ExtensionIdentifier:json!(Request.extension_id),

		Options:Some(json!({ "debugType": Request.debug_type })),
	};

	Service
		.environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.RegisterProvider(Handle, DTO);

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://debug/register",

		json!({ "debugType": Request.debug_type, "extensionId": Request.extension_id }),
	);

	Ok(Response::new(Empty {}))
}
