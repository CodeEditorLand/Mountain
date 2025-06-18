// @module LanguageFeatureLogic
// @description Contains the core logic for managing language feature provider
// registrations.

use Common::{
	error::CommonError,
	language_feature::DTO::{ProviderOptionsDTO, ProviderType},
};
use log::{info, warn};
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime};

use crate::ApplicationState::{ApplicationState::ApplicationState, DTO::ProviderRegistrationDTO};

// Logic to register a provider from Cocoon in `ApplicationState`.
pub async fn RegisterProviderLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	sidecar_identifier:String,
	provider_type:ProviderType,
	selector_DTO:Value,
	extension_identifier_DTO:Value,
	options_DTO:Option<ProviderOptionsDTO>,
) -> Result<u32, CommonError> {
	let app_state = app_handle.state::<ApplicationState>();
	let handle = app_state.GetNextProviderHandle();
	info!(
		"[LangFeatureLogic] Registering {:?} provider from '{}' with new handle {}",
		provider_type, sidecar_identifier, handle
	);

	let new_registration = ProviderRegistrationDTO {
		Handle:handle,
		ProviderType:provider_type,
		Selector:selector_DTO,
		SidecarIdentifier:sidecar_identifier,
		Options:options_DTO,
		ExtensionIdentifier:extension_identifier_DTO,
	};

	let mut providers_map_guard = app_state.LanguageProviders.lock().unwrap();
	providers_map_guard.insert(handle, new_registration);

	Ok(handle)
}

// Logic to unregister a provider from `ApplicationState`.
pub async fn UnregisterProviderLogic<R:Runtime>(app_handle:&AppHandle<R>, handle:u32) -> Result<(), CommonError> {
	info!("[LangFeatureLogic] Unregistering provider with handle {}", handle);
	let app_state = app_handle.state::<ApplicationState>();
	let mut providers_map_guard = app_state.LanguageProviders.lock().unwrap();
	if providers_map_guard.remove(&handle).is_none() {
		warn!(
			"[LangFeatureLogic] Attempted to unregister a provider with handle {} that was not found.",
			handle
		);
	}
	Ok(())
}
