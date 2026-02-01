//! # Configuration Initialize Module
//!
//! Initializes and merges application configurations from multiple sources.

use crate::Environment::MountainEnvironment::MountainEnvironment;
use crate::Environment::ConfigurationProvider::InitializeAndMergeConfigurations;
use log::{error, info};

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
pub async fn ConfigurationInitialize(
	Environment: &std::sync::Arc<MountainEnvironment>,
) -> Result<(), String> {
	match InitializeAndMergeConfigurations(Environment).await {
		Ok(()) => {
			info!("[Config] [Initialize] Configuration initialized and merged successfully.");
			Ok(())
		},
		Err(e) => {
			error!("[Config] [Initialize] Failed: {}", e);
			Err(format!("Failed to initialize and merge configurations: {}", e))
		},
	}
}
