//! # PerformanceStatsCommand
//!
//! Retrieves application performance statistics.
//!
//! ## RESPONSIBILITIES
//!
//! ### Performance Monitoring
//! - Get performance metrics
//! - Report CPU/memory usage
//! - Return operational statistics
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - IPC wrapper command in Binary subsystem
//! - Performance monitoring endpoint
//!
//! ### Dependencies
//! - crate::IPC::AdvancedFeatures: Performance tracking
//! - tauri: IPC framework
//! - serde_json: JSON serialization
//!
//! ### Dependents
//! - Wind frontend: Queries performance stats
//! - DevTools: Performance monitoring
//!
//! ## SECURITY
//!
//! ### Considerations
//! - Stats are read-only, no security impact
//! - Avoid exposing sensitive paths in stats
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - Stats collection should be lightweight
//! - Sampling rate affects accuracy vs overhead

use serde_json::{Value, to_value};
use tauri::AppHandle;

/// Get performance stats.
///
/// Retrieves current performance statistics for monitoring.
///
/// # Arguments
///
/// * `app_handle` - Tauri application handle
///
/// # Returns
///
/// Returns performance stats JSON, or an error string.
///
/// # Errors
///
/// Returns an error if stats cannot be collected.
#[tauri::command]
pub async fn Fn(app_handle:AppHandle) -> Result<Value, String> {
	let stats =
		crate::IPC::AdvancedFeatures::Fn::Fn(app_handle).await?;

	to_value(&stats).map_err(|E| format!("Failed to serialize performance stats: {}", e))
}
