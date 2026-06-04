//! # Route
//!
//! ## File: IPC/Message/Route/RouteMessage.rs
//!
//! ## Role in Mountain Architecture
//!
//! Routes IPC messages to their registered listeners based on channel name,
//! implementing a publish-subscribe pattern for message distribution.
//!
//! ## Primary Responsibility
//!
//! Route incoming IPC messages from Wind to registered listener callbacks
//! based on channel name, supporting both point-to-point and broadcast
//! patterns.
//!
//! ## Secondary Responsibilities
//!
//! - Register new listeners for channels
//! - Remove listeners when no longer needed
//! - Handle listener errors gracefully
//! - Log routing events for debugging
//!
//! ## Dependencies
//!
//! **External Crates:**
//! - `std::collections::HashMap` - Channel-to-listener mapping
//! - `log` - Routing event logging
//!
//! **Internal Modules:**
//! - `DefineMessage::{TauriIPCMessage, ListenerCallback}` - Message and
//!   callback types
//!
//! ## Dependents
//!
//! - `TauriIPCServer` - Uses for message distribution
//! - `HandleIncomingMessage` - Routes messages to listeners
//!
//! ## VSCode Pattern Reference
//!
//! Matches VSCode's channel-based routing in
//! `vs/base/parts/ipc/common/ipc.net.ts`
//! - Channel name mapping to handlers
//! - Multiple listeners per channel support
//! - Listener cleanup on removal
//!
//! ## Security Considerations
//!
//! - Validate channel names to prevent injection
//! - Prevent listener callback errors from crashing server
//! - Limit number of listeners per channel to prevent resource exhaustion
//! - Sanitize listener data before processing
//!
//! ## Performance Considerations
//!
//! - HashMap provides O(1) channel lookup
//! - Lock contention minimized by short critical sections
//! - Error handling doesn't block other listeners
//!
//! ## Error Handling Strategy
//!
//! - Listener errors logged but don't prevent other listeners from receiving
//!   message
//! - Returns Result for explicit error handling
//! - Detailed error messages with channel and listener context
//!
//! ## Thread Safety
//!
//! - HashMap wrapped in Arc<Mutex> for safe concurrent access
//! - Lock contention minimized by short critical sections
//!
//! ## TODO Items
//!
//! - [ ] Add listener priority ordering
//! - [ ] Implement wildcard channel patterns
//! - [ ] Add channel filtering rules

use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};

use super::super::Define::DefineMessage::{ListenerCallback, TauriIPCMessage};

use crate::dev_log;

/// Maximum listeners per channel to prevent resource exhaustion
const MAX_LISTENERS_PER_CHANNEL:usize = 100;

/// Message router for IPC channel-based message distribution
///
/// This router implements a publish-subscribe pattern where listeners can
/// register to receive messages on specific channels.
pub struct Router {

	/// Map from channel names to their registered listeners
	listeners:Arc<Mutex<HashMap<String, Vec<ListenerCallback>>>>,
}

impl Router {

	/// Create a new message router
	///
	/// # Returns
	/// A new Router instance with empty listener map
	pub fn new() -> Self { Self { listeners:Arc::new(Mutex::new(HashMap::new())) } }

	/// Register a listener for a specific channel
	///
	/// # Arguments
	/// * `Channel` - Channel name to listen on
	/// * `Callback` - Callback function to invoke for messages
	///
	/// # Returns
	/// Ok(()) on success, Err with error description on failure
	pub fn Register(&self, Channel:&str, Callback:ListenerCallback) -> Result<(), String> {
		self.validate_channel_name(Channel)?;

		let mut listeners = self
			.listeners
			.lock()
			.map_err(|e| format!("Failed to access listeners: {}", e))?;

		let channel_listeners = listeners.entry(Channel.to_string()).or_insert_with(Vec::new);

		// Check limit before adding
		if channel_listeners.len() >= MAX_LISTENERS_PER_CHANNEL {
			return Err(format!(
				"Maximum listeners ({}) reached for channel: {}",

				MAX_LISTENERS_PER_CHANNEL, Channel
			));
		}

		channel_listeners.push(Callback);

		dev_log!(
			"ipc",

			"[Router] Listener registered for channel: {} (total: {})",

			Channel,

			channel_listeners.len()
		);

		Ok(())
	}

	/// Remove a listener from a channel
	///
	/// # Arguments
	/// * `Channel` - Channel name
	/// * `Callback` - Callback to remove
	///
	/// # Returns
	/// Ok(()) on success, Err with error descriptionon failure
	pub fn Remove(&self, Channel:&str, Callback:&ListenerCallback) -> Result<(), String> {
		let mut listeners = self
			.listeners
			.lock()
			.map_err(|e| format!("Failed to access listeners: {}", e))?;

		if let Some(channel_listeners) = listeners.get_mut(Channel) {
			let initial_count = channel_listeners.len();

			channel_listeners.retain(|cb| !std::ptr::eq(cb as *const _, Callback as *const _));

			let removed_count = initial_count - channel_listeners.len();

			// Clean up empty channels
			if channel_listeners.is_empty() {
				listeners.remove(Channel);

				dev_log!(
					"ipc",

					"[Router] Channel cleaned up: {} (removed {} listeners)",

					Channel,

					removed_count
				);
			} else {
				dev_log!(
					"ipc",

					"[Router] Listener removed from channel: {}, remaining: {}",

					Channel,

					channel_listeners.len()
				);
			}
		} else {
			dev_log!("ipc", "warn: [Router] Channel not found for listener removal: {}", Channel);
		}

		Ok(())
	}

	/// Route a message to all listeners on its channel
	///
	/// # Arguments
	/// * `Message` - Message to route
	///
	/// # Returns
	/// Ok(()) on success, Err with error description on failure
	/// Errors from individual listeners are logged but don't fail the entire
	/// operation
	pub fn RouteMessage(&self, Message:&TauriIPCMessage) -> Result<(), String> {
		dev_log!("ipc", "[Router] Routing message on channel: {}", Message.channel);

		// Validate message before routing
		Message.validate().map_err(|e| format!("Message validation failed: {}", e))?;

		let listeners = self
			.listeners
			.lock()
			.map_err(|e| format!("Failed to access listeners: {}", e))?;

		let listeners_map = &*listeners;

		let channel_listeners = listeners_map.get(&Message.channel);

		if let Some(channel_listeners) = channel_listeners {
			let listener_count = channel_listeners.len();

			let mut success_count = 0;

			let mut error_count = 0;

			for (index, callback) in channel_listeners.iter().enumerate() {
				let message_data = Message.data.clone();

				match callback(message_data) {
					Ok(_) => success_count += 1,

					Err(e) => {
						dev_log!(
							"ipc",

							"error: [Router] Error in listener {} for channel {}: {}",

							index,

							Message.channel,

							e
						);

						error_count += 1;
					},
				}
			}

			dev_log!(
				"ipc",

				"[Router] Message routed to channel {}: {}/{} listeners succeeded",

				Message.channel,

				success_count,

				listener_count
			);

			if error_count > 0 {
				dev_log!(
					"ipc",

					"warn: [Router] {} listener(s) failed on channel {}",

					error_count,

					Message.channel
				);
			}
		} else {
			dev_log!("ipc", "[Router] No listeners found for channel: {}", Message.channel);
		}

		Ok(())
	}

	/// Get all registered channels
	///
	/// # Returns
	/// Ok(Vec<String>) with all channel names
	pub fn GetChannels(&self) -> Result<Vec<String>, String> {
		let listeners = self
			.listeners
			.lock()
			.map_err(|e| format!("Failed to access listeners: {}", e))?;

		Ok(listeners.keys().cloned().collect())
	}

	/// Get listener count for a specific channel
	///
	/// # Arguments
	/// * `Channel` - Channel name
	///
	/// # Returns
	/// Ok(usize) listener count or Err with error
	pub fn GetListenerCount(&self, Channel:&str) -> Result<usize, String> {
		let listeners = self
			.listeners
			.lock()
			.map_err(|e| format!("Failed to access listeners: {}", e))?;

		Ok(listeners.get(Channel).map_or(0, |l| l.len()))
	}

	/// Clear all listeners for a channel
	///
	/// # Arguments
	/// * `Channel` - Channel to clear
	///
	/// # Returns
	/// Ok(()) on success, Err with error description
	pub fn ClearChannel(&self, Channel:&str) -> Result<(), String> {
		self.validate_channel_name(Channel)?;

		let mut listeners = self
			.listeners
			.lock()
			.map_err(|e| format!("Failed to access listeners: {}", e))?;

		let count = listeners.get(Channel).map_or(0, |l| l.len());

		listeners.remove(Channel);

		dev_log!("ipc", "[Router] Cleared {} listeners from channel: {}", count, Channel);

		Ok(())
	}

	/// Clear all listeners from all channels
	///
	/// # Returns
	/// Ok(()) on success, Err with error description
	pub fn ClearAll(&self) -> Result<(), String> {
		let mut listeners = self
			.listeners
			.lock()
			.map_err(|e| format!("Failed to access listeners: {}", e))?;

		let total_listeners:usize = listeners.values().map(|l| l.len()).sum();

		listeners.clear();

		dev_log!(
			"ipc",

			"[Router] Cleared {} listeners from {} channels",

			total_listeners,

			listeners.len()
		);

		Ok(())
	}

	/// Validate channel name
	///
	/// # Arguments
	/// * `Channel` - Channel name to validate
	///
	/// # Returns
	/// Ok(()) if valid, Err with error description
	fn validate_channel_name(&self, Channel:&str) -> Result<(), String> {
		if Channel.is_empty() {
			return Err("Channel name cannot be empty".to_string());
		}

		// Check for valid characters (alphanumeric, hyphen, underscore, colon)
		if !Channel.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ':') {
			return Err(format!(
				"Channel contains invalid characters: '{}' (only alphanumeric, -, _, : allowed)",

				Channel
			));
		}

		// Reasonable length limit
		if Channel.len() > 256 {
			return Err(format!("Channel name too long: {} (max 256)", Channel.len()));
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {

	use serde_json::json;

	use super::*;

	fn create_test_callback(response:&str) -> ListenerCallback {
		let response = response.to_string();

		Box::new(move |_:serde_json::Value| Ok::<(), String>(response.clone()))
	}

	#[test]
	fn test_register_and_route() {
		let router = Router::new();

		let callback = create_test_callback("received");

		router.Register("test-channel", callback).expect("Registration failed");

		let message = TauriIPCMessage::new("test-channel", json!({"test": true}), None);

		router.RouteMessage(&message).expect("Routing failed");
	}

	#[test]
	fn test_channel_validation() {
		let router = Router::new();

		let callback = create_test_callback("ok");

		// Empty channel
		assert!(router.Register("", callback).is_err());

		// Invalid characters
		assert!(router.Register("test channel", callback).is_err());

		// Valid channels
		assert!(router.Register("test-channel", callback).is_ok());

		assert!(router.Register("test_channel", callback).is_ok());

		assert!(router.Register("test:channel", callback).is_ok());
	}

	#[test]
	fn test_listener_limit() {
		let router = Router::new();

		let callback = create_test_callback("ok");

		// Register up to limit
		for i in 0..MAX_LISTENERS_PER_CHANNEL {
			let cb = create_test_callback(&format!("listener{}", i));

			assert!(router.Register("test", cb.clone()).is_ok());
		}

		// One more should fail
		assert!(router.Register("test", callback).is_err());
	}

	#[test]
	fn test_get_listener_count() {
		let router = Router::new();

		let callback = create_test_callback("ok");

		assert_eq!(router.GetListenerCount("test").unwrap(), 0);

		router.Register("test", callback).unwrap();

		assert_eq!(router.GetListenerCount("test").unwrap(), 1);

		router.Register("test", create_test_callback("ok")).unwrap();

		assert_eq!(router.GetListenerCount("test").unwrap(), 2);
	}

	#[test]
	fn test_clear_channel() {
		let router = Router::new();

		let callback = create_test_callback("ok");

		router.Register("test", callback).unwrap();

		router.Register("test", create_test_callback("ok")).unwrap();

		assert_eq!(router.GetListenerCount("test").unwrap(), 2);

		router.ClearChannel("test").unwrap();

		assert_eq!(router.GetListenerCount("test").unwrap(), 0);
	}
}
