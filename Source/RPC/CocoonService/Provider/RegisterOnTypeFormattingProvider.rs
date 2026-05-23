
//! Register a Cocoon-side on-type-formatting provider. Uses the
//! type-formatting-specific request shape (carries trigger characters).

use tonic::{Response, Status};
use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, RegisterOnTypeFormattingProviderRequest},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:RegisterOnTypeFormattingProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering On Type Formatting Provider");

	Service.RegisterProvider(
		Request.handle,
		ProviderType::OnTypeFormatting,
		&Request.language_selector,
		&Request.extension_id,
	);

	Ok(Response::new(Empty {}))
}
