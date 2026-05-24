//! Register a Cocoon-side signature-help provider. Uses the
//! signature-help-specific request shape (carries trigger characters).

use tonic::{Response, Status};
use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, RegisterSignatureHelpProviderRequest},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:RegisterSignatureHelpProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering Signature Help Provider");

	Service.RegisterProvider(
		Request.handle,
		ProviderType::SignatureHelp,
		&Request.language_selector,
		&Request.ExtensionId,
	);

	Ok(Response::new(Empty {}))
}
