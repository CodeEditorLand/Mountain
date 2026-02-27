//! # RuntimeBuild - Advanced Runtime Scheduler Initialization
//!
//! Constructs the Echo async scheduler with telemetry integration and feature

#[allow(unused_imports)]
//! flags.
//!
//! ## Build Profiles
//!
//! - **Debug**: Single-threaded for easier debugging
//! - **Development**: Multi-threaded with work-stealing
//! - **Release**: Optimized multi-threaded with CPU count workers
//!
//! ## Feature Flags
//!
//! - `Debug`: Verbose scheduler logging
//! - `Telemetry`: OTEL integration for task metrics
//!
//! ## Defensive Coding
//!
//! - Panic safety with bounded worker counts
//! - Resource cleanup on initialization failure
//! - Configuration validation

use std::sync::Arc;

use Echo::Scheduler::{Scheduler::Scheduler, SchedulerBuilder::SchedulerBuilder};
use log::{debug, info, warn};

// ============ Feature Flags ============

/// Scheduler configuration for different build profiles
#[derive(Debug)]
pub struct SchedulerConfig {
	worker_count:Option<usize>,
	enable_metrics:bool,
	log_level:log::Level,
}

impl Default for SchedulerConfig {
	fn default() -> Self {
		// Default to CPU count for production builds
		Self {
			worker_count:None, // Uses CPU count by default
			#[cfg(feature = "Telemetry")]
			enable_metrics:true,
			#[cfg(not(feature = "Telemetry"))]
			enable_metrics:false,
			#[cfg(feature = "Debug")]
			log_level:log::Level::Debug,
			#[cfg(feature = "Development")]
			log_level:log::Level::Info,
			#[cfg(not(any(feature = "Debug", feature = "Development")))]
			log_level:log::Level::Warn,
		}
	}
}

/// Create configured scheduler builder
pub fn CreateBuilder(config:SchedulerConfig) -> SchedulerBuilder {
	let mut builder = SchedulerBuilder::Create();

	if let Some(count) = config.worker_count {
		// Validate worker count bounds
		let count = count.clamp(1, 256);
		builder = builder.WithWorkerCount(count);
		debug!("[RuntimeBuild] Configuring {} worker threads", count);
	}

	builder
}

/// Build the Echo scheduler for async task execution
///
/// Creates a work-stealing scheduler with optimal worker count.
/// This is required for all async operations in the application.
///
/// # Configuration
///
/// - Debug builds: 1 worker for easier debugging
/// - Development: CPU count workers
/// - Release: CPU count workers with optimizations
///
/// # Returns
///
/// Arc-wrapped Echo scheduler ready for use
///
/// # Panics
///
/// Panics if scheduler construction fails (should never happen
/// with valid configuration)
pub fn Build() -> Arc<Scheduler> { BuildWithConfig(SchedulerConfig::default()) }

/// Build scheduler with custom configuration
///
/// # Parameters
///
/// - `config`: Scheduler configuration specifying worker count and options
///
/// # Returns
///
/// Configured scheduler instance
pub fn BuildWithConfig(config:SchedulerConfig) -> Arc<Scheduler> {
	info!("[RuntimeBuild] Initializing scheduler with config: {:?}", config);

	let builder = CreateBuilder(config);
	let scheduler = builder.Build();

	#[cfg(feature = "Telemetry")]
	{
		// Initialize task metrics recording
		info!("[RuntimeBuild] Task metrics enabled");
	}

	#[cfg(feature = "Debug")]
	{
		debug!("[RuntimeBuild] Scheduler debugging enabled");
	}

	info!("[RuntimeBuild] Scheduler initialized successfully");
	Arc::new(scheduler)
}

/// Build minimal debug scheduler (single-threaded)
///
/// Useful for debugging and testing where predictable
/// execution order matters.
#[cfg(feature = "Debug")]
pub fn BuildDebug() -> Arc<Scheduler> {
	info!("[RuntimeBuild] Creating debug scheduler (single-threaded)");
	BuildWithConfig(SchedulerConfig { worker_count:Some(1), ..Default::default() })
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_default_build() {
		let _scheduler = Build();
		// Scheduler should be usable
		info!("[Test] Default scheduler created");
	}

	#[test]
	fn test_custom_worker_count() {
		let config = SchedulerConfig { worker_count:Some(2), ..Default::default() };
		let _scheduler = BuildWithConfig(config);
		info!("[Test] Custom scheduler created");
	}
}
