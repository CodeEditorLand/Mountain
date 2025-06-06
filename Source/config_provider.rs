// 
mod InternalUtils {
	// ... other utils ...

	pub mod ConfigProviderImpl {
		use super::*; // To get access to parent's imports like CommonError, Handlers, etc.

		pub async fn GetConfigurationValue(
			EnvironmentInstance:&MountainEnvironment, // Pass &MountainEnvironment
			SectionKeyOption:Option<String>,
			Overrides:IConfigurationOverrides,
		) -> Result<Value, CommonError> {
			trace!(
				"[MountainEnvironment ConfigProviderImpl] GetConfig: section={:?}, overrides.resource={:?}, \
				 overrides.langId={:?}",
				SectionKeyOption,
				Overrides.Resource.as_ref().and_then(|v| v.get("external")),
				Overrides.OverrideIdentifier
			);
			let AppStateInstance = EnvironmentInstance.GetAppState(); // Use helper
			let ConfigStateGuard = AppStateInstance
				.Configuration
				.lock()
				.map_err(MapAppStateLockErrorToCommonError)?; // Use helper
			// ... rest of original logic from config_provider.rs ...
			let ValueResult = ConfigStateGuard.get_value(SectionKeyOption.as_deref(), Overrides.Resource.as_ref());
			debug!(
				"[MountainEnvironment ConfigProviderImpl GetConfig] Value for section {:?}: (sample) {}...",
				SectionKeyOption,
				ValueResult.to_string().chars().take(70).collect::<String>()
			);
			Ok(ValueResult)
		}

		pub async fn UpdateConfigurationValue(
			EnvironmentInstance:&MountainEnvironment,
			KeyToUpdate:String,
			ValueToSet:Value,
			TargetScope:ConfigurationTarget,
			Overrides:IConfigurationOverrides,
			ScopeToLanguageOverride:Option<bool>,
		) -> Result<(), CommonError> {
			// ... logic from config_provider.rs, calling Handlers::Config ...
			// e.g., Handlers::Config::GetConfigPathForTarget(&EnvironmentInstance.
			// AppHandle, ...)
			info!("[MountainEnvironment ConfigProviderImpl UpdateConfig] ... ");
			let AppStateInstance = EnvironmentInstance.GetAppState();
			let TargetConfigFilePath = Handlers::Config::GetConfigPathForTarget(
				// Assuming Handlers::Config also gets PascalCased
				&EnvironmentInstance.AppHandle,
				&AppStateInstance,
				TargetScope,
				&Overrides,
				ScopeToLanguageOverride.unwrap_or(false),
			)?;
			// ... rest of the logic ...
			Ok(())
		}

		pub async fn InspectConfigurationValue(
			EnvironmentInstance:&MountainEnvironment,
			Key:String,
			Overrides:IConfigurationOverrides,
		) -> Result<Option<InspectResultData>, CommonError> {
			// ... logic from config_provider.rs ...
			Ok(None) // Placeholder
		}
	}
	// ... OtherProviderImpl modules ...
}
