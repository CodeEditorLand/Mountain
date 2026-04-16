//! # Scheduler Shutdown Module
//!
//! Handles graceful shutdown of the Echo task scheduler.

#[allow(unused_imports)]
use std::sync::Arc;

use Echo::Scheduler::Scheduler::Scheduler;
use crate::dev_log;

/// Stops the Echo task scheduler and cleans up its resources.
///
/// # Arguments
///
/// * `SchedulerForShutdown` - Arc-wrapped scheduler to shut down
///
/// # Returns
///
/// A `Result` indicating success or failure.
///
/// # Shutdown Process
///
/// This function performs:
/// - Stops accepting new tasks
/// - Completes in-progress tasks
/// - Cleans up scheduler resources
///
/// # Errors
///
/// Returns an error if the scheduler is not exclusively owned or stop fails.
pub async fn SchedulerShutdown(SchedulerForShutdown:Arc<Scheduler>) -> Result<(), String> {
	dev_log!("lifecycle", "[Shutdown] [Scheduler] Stopping Echo scheduler...");

	// Try to get exclusive ownership for shutdown
	match Arc::try_unwrap(SchedulerForShutdown) {
		Ok(mut Scheduler) => {
			Scheduler.Stop().await;
			dev_log!("lifecycle", "[Shutdown] [Scheduler] Echo scheduler stopped successfully.");
			Ok(())
		},
		Err(_) => Err("Scheduler not exclusively owned".to_string()),
	}
}
