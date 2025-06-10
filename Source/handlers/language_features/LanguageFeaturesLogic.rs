use Common::{
	error::CommonError,
	language_feature::dto::{ProviderOptionsDto, ProviderType},
};
use log::{info, warn};
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime};

/// @module LanguageFeaturesLogic
/// @description Contains the core logic for managing language feature provider
/// registrations.
use crate::AppState::{AppState::AppState, Dto::ProviderRegistrationDto};

/// Logic to register a provider from Cocoon in `AppState`.
pub async fn RegisterProviderLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	SidecarIdentifier:String,
	ProviderType:ProviderType,
	SelectorDto:Value,
	ExtensionIdentifierDto:Value,
	OptionsDto:Option<ProviderOptionsDto>,
) -> Result<u32, CommonError> {
	let AppStateInstance = AppHandle.state::<AppState>();
	let Handle = AppStateInstance.GetNextProviderHandle();
	info!(
		"[LangFeaturesLogic] Registering {:?} provider from '{}' with new handle {}",
		ProviderType, SidecarIdentifier, Handle
	);

	let NewRegistration = ProviderRegistrationDto {
		Handle,
		ProviderType,
		Selector:SelectorDto,
		SidecarIdentifier,
		Options:OptionsDto,
		ExtensionIdentifier:ExtensionIdentifierDto,
	};

	let mut ProvidersMapGuard = AppStateInstance.LanguageProviders.lock().unwrap();
	ProvidersMapGuard.insert(Handle, NewRegistration);

	Ok(Handle)
}

/// Logic to unregister a provider from `AppState`.
pub async fn UnregisterProviderLogic<R:Runtime>(AppHandle:&AppHandle<R>, Handle:u32) -> Result<(), CommonError> {
	info!("[LangFeaturesLogic] Unregistering provider with handle {}", Handle);
	let AppStateInstance = AppHandle.state::<AppState>();
	let mut ProvidersMapGuard = AppStateInstance.LanguageProviders.lock().unwrap();
	ProvidersMapGuard.remove(&Handle);
	Ok(())
}

// NOTE: The logic for invoking providers (e.g., `ProvideHoverLogic`) would now
// be implemented in the `Support/` directory and would involve:
// 1. Finding the correct provider in `AppState.LanguageProviders`.
// 2. Making a gRPC call to the provider's `SidecarIdentifier`.
// 3. Returning the result.
