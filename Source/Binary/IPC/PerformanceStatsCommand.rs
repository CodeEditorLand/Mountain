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

use serde_json::Value;
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
pub async fn MountainGetPerformanceStats(app_handle: AppHandle) -> Result<Value, String> {
	crate::IPC::AdvancedFeatures::mountain_get_performance_stats(app_handle).await
}
