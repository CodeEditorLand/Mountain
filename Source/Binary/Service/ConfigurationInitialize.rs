//! # Configuration Initialize Module
//!
//! Initializes and merges application configurations from multiple sources.


use crate::Environment::{
use crate::dev_log;
	ConfigurationProvider::Loading::initialize_and_merge_configurations,
	MountainEnvironment::MountainEnvironment,
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
///
/// # Configuration Sources
///
/// The configuration initialization process merges configurations from:
/// - Default application settings
/// - User configuration files
/// - Environment variables
/// - Command-line arguments
/// - External service configurations
///
/// # Errors
///
/// Returns an error if configuration initialization or merging fails.
pub async fn ConfigurationInitialize(Environment:&std::sync::Arc<MountainEnvironment>) -> Result<(), String> {
	match initialize_and_merge_configurations(Environment).await {
		Ok(()) => {
			dev_log!("config", "[Config] [Initialize] Configuration initialized and merged successfully.");
			Ok(())
		},
		Err(e) => {
			dev_log!("config", "error: [Config] [Initialize] Failed: {}", e);
			Err(format!("Failed to initialize and merge configurations: {}", e))
		},
	}
}
