//! Provider registration and unregistration logic.

use CommonLibrary::{Error::CommonError::CommonError, LanguageFeature::DTO::ProviderType::ProviderType};
use serde_json::Value;

use crate::{
	ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO,
	Environment::Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError,
	dev_log,
};

pub(super) async fn register_provider(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	side_car_identifier:String,

	provider_type:ProviderType,

	selector_dto:Value,

	extension_identifier_dto:Value,

	options_dto:Option<Value>,
) -> Result<u32, CommonError> {
	let handle = environment.ApplicationState.GetNextProviderHandle();

	let new_registration = ProviderRegistrationDTO {
		Handle:handle,

		ProviderType:provider_type,

		Selector:selector_dto,

		SideCarIdentifier:side_car_identifier,

		ExtensionIdentifier:extension_identifier_dto,

		Options:options_dto,
	};

	environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.LanguageProviders
		.lock()
		.map_err(MapApplicationStateLockErrorToCommonError)?
		.insert(handle, new_registration);

	Ok(handle)
}

pub(super) async fn unregister_provider(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	handle:u32,
) -> Result<(), CommonError> {
	let mut providers = environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.LanguageProviders
		.lock()
		.map_err(MapApplicationStateLockErrorToCommonError)?;

	if providers.remove(&handle).is_none() {
		dev_log!(
			"extensions",
			"warn: Attempted to unregister non-existent provider handle: {}",
			handle
		);
	}

	Ok(())
}
