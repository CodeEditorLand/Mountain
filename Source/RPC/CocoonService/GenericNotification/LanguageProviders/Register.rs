
//! Register a language feature provider by handle, selector, extension.

use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;
use serde_json::Value;

use crate::RPC::CocoonService::CocoonServiceImpl;

pub fn Fn(Params:Value, Service:&CocoonServiceImpl, ProvType:ProviderType) {
	let Handle = Params.get("handle").and_then(|V| V.as_u64()).unwrap_or(0) as u32;

	let Selector = Params.get("language_selector").and_then(|V| V.as_str()).unwrap_or("*");

	let ExtId = Params.get("extension_id").and_then(|V| V.as_str()).unwrap_or("");

	Service.RegisterProvider(Handle, ProvType, Selector, ExtId);
}
