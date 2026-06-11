//! Registers a debug configuration provider (config resolver) for a debug
//! type, recording its handle and owning sidecar in `ApplicationState`.

use CommonLibrary::Error::CommonError::CommonError;

use crate::{Environment::MountainEnvironment::MountainEnvironment, dev_log};

pub(crate) async fn Fn(
	Environment:&MountainEnvironment,

	DebugType:String,

	ProviderHandle:u32,

	SideCarIdentifier:String,
) -> Result<(), CommonError> {
	// Validate debug type is non-empty
	if DebugType.is_empty() {
		return Err(CommonError::InvalidArgument {
			ArgumentName:"DebugType".to_string(),
			Reason:"DebugType cannot be empty".to_string(),
		});
	}

	dev_log!(
		"exthost",
		"[DebugProvider] Registering DebugConfigurationProvider for type '{}' (handle: {}, sidecar: {})",
		DebugType,
		ProviderHandle,
		SideCarIdentifier
	);

	// Store debug configuration provider registration in ApplicationState
	Environment
		.ApplicationState
		.Feature
		.Debug
		.RegisterDebugConfigurationProvider(DebugType, ProviderHandle, SideCarIdentifier)
		.map_err(|e| CommonError::Unknown { Description:e })?;

	Ok(())
}
