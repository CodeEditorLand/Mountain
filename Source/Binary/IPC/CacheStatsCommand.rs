//! # CacheStatsCommand
//!
//! Retrieves cache statistics for monitoring.
//!
//! ## RESPONSIBILITIES
//!
//! ### Cache Monitoring
//! - Get cache metrics
//! - Report cache hit/miss rates
//! - Return memory usage statistics
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - IPC wrapper command in Binary subsystem
//! - Cache monitoring endpoint
//!
//! ### Dependencies
//! - crate::IPC::AdvancedFeatures: Cache tracking
//! - tauri: IPC framework
//! - serde_json: JSON serialization
//!
//! ### Dependents
//! - Wind frontend: Queries cache stats
//! - DevTools: Performance monitoring
//!
//! ## SECURITY
//!
//! ### Considerations
//! - Stats are read-only, no security impact
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - Stats collection should be lightweight
//! - Update interval affects accuracy

use serde_json::{Value, to_value};
use tauri::AppHandle;

/// Get cache stats.
///
/// Retrieves cache performance statistics for monitoring.
///
/// # Arguments
///
/// * `app_handle` - Tauri application handle
///
/// # Returns
///
/// Returns cache stats JSON, or an error string.
///
/// # Errors
///
/// Returns an error if stats cannot be collected.
#[tauri::command]
pub async fn MountainGetCacheStats(app_handle:AppHandle) -> Result<Value, String> {
	let stats = crate::IPC::AdvancedFeatures::mountain_get_cache_stats(app_handle).await?;
	to_value(&stats).map_err(|e| format!("Failed to serialize cache stats: {}", e))
}
