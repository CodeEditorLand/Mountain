//! # OutputChannelState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages output channel state including output metadata, content, and
//! presentation state.
//!
//! ## ARCHITECTURAL ROLE
//! OutputChannelState is part of the **FeatureState** module, representing
//! output channel state organized by channel ID.
//!
//! ## KEY COMPONENTS
//! - OutputChannelState: Main struct containing output channels map
//! - Default: Initialization implementation
//! - Helper methods: Output channel manipulation utilities
//!
//! ## ERROR HANDLING
//! - Thread-safe access via `Arc<Mutex<...>>`
//! - Proper lock error handling with `MapLockError` helpers
//!
//! ## LOGGING
//! State changes are logged at appropriate levels (debug, info, warn, error).
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Lock mutexes briefly and release immediately
//! - Avoid nested locks to prevent deadlocks
//! - Use Arc for shared ownership across threads
//!
//! ## TODO
//! - [ ] Add output channel validation invariants
//! - [ ] Implement output channel lifecycle events
//! - [ ] Add output channel metrics collection

use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};

use crate::{ApplicationState::DTO::OutputChannelStateDTO::OutputChannelStateDTO, dev_log};

/// Output channels state containing channels by ID.
#[derive(Clone)]
pub struct OutputChannelState {

	/// Output channels organized by ID.
	pub OutputChannels:Arc<StandardMutex<HashMap<String, OutputChannelStateDTO>>>,
}

impl Default for OutputChannelState {

	fn default() -> Self {

		dev_log!("output", "[OutputChannelState] Initializing default output channel state...");

		Self { OutputChannels:Arc::new(StandardMutex::new(HashMap::new())) }
	}
}

impl OutputChannelState {

	/// Gets all output channels.
	pub fn GetAll(&self) -> HashMap<String, OutputChannelStateDTO> {

		self.OutputChannels.lock().ok().map(|guard| guard.clone()).unwrap_or_default()
	}

	/// Gets an output channel by its ID.
	pub fn Get(&self, id:&str) -> Option<OutputChannelStateDTO> {

		self.OutputChannels.lock().ok().and_then(|guard| guard.get(id).cloned())
	}

	/// Adds or updates an output channel.
	pub fn AddOrUpdate(&self, id:String, channel:OutputChannelStateDTO) {

		if let Ok(mut guard) = self.OutputChannels.lock() {

			guard.insert(id, channel);

			dev_log!("output", "[OutputChannelState] Output channel added/updated");
		}
	}

	/// Removes an output channel by its ID.
	pub fn Remove(&self, id:&str) {

		if let Ok(mut guard) = self.OutputChannels.lock() {

			guard.remove(id);

			dev_log!("output", "[OutputChannelState] Output channel removed: {}", id);
		}
	}

	/// Clears all output channels.
	pub fn Clear(&self) {

		if let Ok(mut guard) = self.OutputChannels.lock() {

			guard.clear();

			dev_log!("output", "[OutputChannelState] All output channels cleared");
		}
	}

	/// Gets the count of output channels.
	pub fn Count(&self) -> usize { self.OutputChannels.lock().ok().map(|guard| guard.len()).unwrap_or(0) }

	/// Checks if an output channel exists.
	pub fn Contains(&self, id:&str) -> bool {

		self.OutputChannels
			.lock()
			.ok()
			.map(|guard| guard.contains_key(id))
			.unwrap_or(false)
	}
}
