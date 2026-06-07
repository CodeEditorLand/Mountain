//! Register a Cocoon SCM provider in `ApplicationState` AND route through
//! the `SourceControlManagementProvider` trait so SCM state is materialised
//! in `ApplicationState::SourceControl` (the surface Sky's SCM view binds
//! to). The prior direct Sky emit bypassed state tracking - providers
//! registered by gitlens/svn/etc. never appeared in the SCM view until a
//! `UpdateScmGroup` call landed.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};
use CommonLibrary::{
	LanguageFeature::DTO::ProviderType::ProviderType,
	SourceControlManagement::SourceControlManagementProvider::SourceControlManagementProvider,
};
use ::Vine::Generated::{Empty, RegisterScmProviderRequest};

use crate::{
	ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO,
	RPC::CocoonService::CocoonServiceImpl,
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:RegisterScmProviderRequest) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering SCM provider: {}", Request.scm_id);

	let Handle = Request
		.scm_id
		.as_bytes()
		.iter()
		.fold(0u32, |Acc, B| Acc.wrapping_mul(31).wrapping_add(*B as u32));

	let DTO = ProviderRegistrationDTO {
		Handle,

		ProviderType:ProviderType::SourceControl,

		Selector:json!([{ "scmId": Request.scm_id }]),

		SideCarIdentifier:"cocoon-main".to_string(),

		ExtensionIdentifier:json!(Request.extension_id),

		Options:Some(json!({ "scmId": Request.scm_id })),
	};

	Service
		.environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.RegisterProvider(Handle, DTO);

	let CreateData = json!({
		"handle": Handle,
		"id": Request.scm_id,
		"label": Request.scm_id,
		"rootUri": null,
		"extensionId": Request.extension_id,
	});

	if let Err(Error) = Service.environment.CreateSourceControl(CreateData).await {
		dev_log!(
			"cocoon",
			"warn: [CocoonService] CreateSourceControl trait failed ({}); falling back to Sky emit",
			Error
		);

		let _ = Service.environment.ApplicationHandle.emit(
			"sky://scm/register",
			json!({ "scmId": Request.scm_id, "extensionId": Request.extension_id }),
		);
	}

	Ok(Response::new(Empty {}))
}
