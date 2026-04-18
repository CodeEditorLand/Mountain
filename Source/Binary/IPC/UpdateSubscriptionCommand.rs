//! # UpdateSubscriptionCommand
//!
//! Manages update subscriptions for document synchronization.
//!
//! ## RESPONSIBILITIES
//!
//! ### Subscription Management
//! - Subscribe to document updates
//! - Manage subscription targets
//! - Track subscriber information
//! - Validate subscription parameters
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - IPC wrapper command in Binary subsystem
//! - Update subscription endpoint
//!
//! ### Dependencies
//! - crate::IPC::WindAdvancedSync: Subscription management
//! - tauri: IPC framework
//! - serde_json: JSON serialization
//! - log: Logging framework
//!
//! ### Dependents
//! - Wind frontend: Subscribes to updates
//!
//! ## SECURITY
//!
//! ### Considerations
//! - Validate target and subscriber identifiers
//! - Implement authorization for subscriptions
//! - Prevent duplicate subscriptions
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - Subscription operations should be fast
//! - Consider batching for bulk subscriptions

use serde_json::Value;
use tauri::AppHandle;

use crate::dev_log;

/// Subscribe to updates.
///
/// Subscribes a subscriber to receive updates for a target.
///
/// # Arguments
///
/// * `app_handle` - Tauri application handle
/// * `subscription_data` - JSON object with target and subscriber fields
///
/// # Returns
///
/// Returns success JSON or an error string.
///
/// # Errors
///
/// Returns an error if:
/// - Required fields missing
/// - Subscription fails
#[tauri::command]
pub async fn MountainSubscribeToUpdates(app_handle:AppHandle, subscription_data:Value) -> Result<Value, String> {
	let Target = subscription_data["target"]
		.as_str()
		.ok_or_else(|| {
			dev_log!("ipc", "error: [IPC] [Sync] Missing target in subscription_data");
			"Missing target"
		})?
		.to_string();
	let Subscriber = subscription_data["subscriber"]
		.as_str()
		.ok_or_else(|| {
			dev_log!("ipc", "error: [IPC] [Sync] Missing subscriber in subscription_data");
			"Missing subscriber"
		})?
		.to_string();

	crate::IPC::WindAdvancedSync::mountain_subscribe_to_updates(app_handle, Target, Subscriber)
		.await
		.map_err(|Error| {
			dev_log!("ipc", "error: [IPC] [Sync] Failed to subscribe to updates: {}", Error);
			Error.to_string()
		})
		.map(|_| Value::Null)
}
