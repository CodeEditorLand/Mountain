//! Register a Cocoon-contributed debug adapter in `ApplicationState` and
//! notify Sky so the debug-launcher UI can light up.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};
use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;

use crate::{
	ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO,
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, RegisterDebugAdapterRequest},
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:RegisterDebugAdapterRequest) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering debug adapter: {}", Request.DebugType);

	let Handle = Request
		.DebugType
		.as_bytes()
		.iter()
		.fold(0u32, |Acc, B| Acc.wrapping_mul(31).wrapping_add(*B as u32));

	let DTO = ProviderRegistrationDTO {
		Handle,

		ProviderType:ProviderType::DebugAdapter,

		Selector:json!([{ "debugType": Request.DebugType }]),

		SideCarIdentifier:"cocoon-main".to_string(),

		ExtensionIdentifier:json!(Request.ExtensionId),

		Options:Some(json!({ "debugType": Request.DebugType })),
	};

	Service
		.environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.RegisterProvider(Handle, DTO);

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://debug/register",
		json!({ "debugType": Request.DebugType, "extensionId": Request.ExtensionId }),
	);

	Ok(Response::new(Empty {}))
}
