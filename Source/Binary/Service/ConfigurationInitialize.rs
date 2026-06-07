// Configuration Initialize Module - Initializes and merges application
// configurations.

use crate::{
	Environment::{
		ConfigurationProvider::Loading::Fn as initialize_and_merge_configurations,
		MountainEnvironment::MountainEnvironment,
	},
	dev_log,
};

/// Initializes and merges all application configurations.
///
/// # Arguments
///
/// * `Environment` - The Mountain environment instance
///
/// # Returns
///
/// A `Result` indicating success or failure.
pub async fn Fn(Environment:&std::sync::Arc<MountainEnvironment>) -> Result<(), String> {

	match initialize_and_merge_configurations(Environment).await {
		Ok(()) => {
			dev_log!(
				"config",

				"[Config] [Initialize] Configuration initialized and merged successfully."
			);

			Ok(())
		},

		Err(e) => {
			dev_log!("config", "error: [Config] [Initialize] Failed: {}", e);

			Err(format!("Failed to initialize and merge configurations: {}", e))
		},
	}
}
