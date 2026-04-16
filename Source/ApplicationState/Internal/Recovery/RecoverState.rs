//! # RecoverState Module (Internal)
//!
//! ## RESPONSIBILITIES
//! Provides state recovery utilities including validation, timeout handling,
//! and exponential backoff for recovery operations.
//!
//! ## ARCHITECTURAL ROLE
//! RecoverState is part of the **Internal::Recovery** module, providing
//! recovery utilities for corrupted or invalid state.
//!
//! ## KEY COMPONENTS
//! - validate_and_clean_state: Filters state by validator function
//! - safe_state_operation_with_timeout: Executes operation with timeout
//! - recover_state_with_backoff: Retries with exponential backoff
//!
//! ## ERROR HANDLING
//! - Validates state before operations
//! - Timeout protection for operations
//! - Exponential backoff for retries
//!
//! ## LOGGING
//! Operations are logged at appropriate levels (error, warn).
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Efficient validation with retain
//! - Timeout prevents hanging operations
//! - Exponential backoff prevents overwhelming system
//!
//! ## TODO
//! - [ ] Add state validation rules
//! - [ ] Implement checkpoint recovery
//! - [ ] Add recovery metrics collection

use std::collections::HashMap;

use CommonLibrary::Error::CommonError::CommonError;
use crate::dev_log;

/// Validates and cleans up state data by removing entries that don't pass
/// validation.
///
/// # Arguments
/// * `state_data` - The state data to validate and clean
/// * `validator` - Function that returns true for valid entries
///
/// # Type Parameters
/// * `T` - The type of values in the state map
///
/// # Behavior
/// - Retains only entries where validator returns true
/// - In-place modification of the HashMap
pub fn validate_and_clean_state<T>(state_data:&mut HashMap<String, T>, validator:impl Fn(&T) -> bool) {
	let original_len = state_data.len();
	state_data.retain(|_, value| validator(value));
	let removed_count = original_len - state_data.len();

	if removed_count > 0 {
		dev_log!("lifecycle", "warn: [RecoverState] Removed {} invalid state entries ({} remaining)",
			removed_count,
			state_data.len());
	}
}

/// Safe state operation with timeout protection.
///
/// # Arguments
/// * `operation` - The operation to execute
/// * `timeout_ms` - Timeout in milliseconds
/// * `operation_name` - Name of the operation for logging
///
/// # Type Parameters
/// * `T` - The return type of the operation
/// * `F` - The operation function type
///
/// # Returns
/// Result containing the operation result or CommonError
///
/// # Behavior
/// - Executes operation in a separate thread
/// - Waits for result or timeout
/// - Returns error if timeout occurs
pub fn safe_state_operation_with_timeout<T, F>(
	operation:F,
	timeout_ms:u64,
	operation_name:&str,
) -> Result<T, CommonError>
where
	F: FnOnce() -> Result<T, CommonError> + Send + 'static,
	T: Send + 'static, {
	let (sender, receiver) = std::sync::mpsc::channel();

	std::thread::spawn(move || {
		let result = operation();
		let _ = sender.send(result);
	});

	match receiver.recv_timeout(std::time::Duration::from_millis(timeout_ms)) {
		Ok(result) => result,
		Err(_) => {
			dev_log!("lifecycle", "error: [RecoverState] Operation '{}' timed out after {}ms", operation_name, timeout_ms);
			Err(CommonError::Unknown { Description:format!("Operation '{}' timed out", operation_name) })
		},
	}
}

/// Attempt state recovery with exponential backoff.
///
/// # Arguments
/// * `operation` - The operation to retry
/// * `max_attempts` - Maximum number of retry attempts
/// * `operation_name` - Name of the operation for logging
///
/// # Type Parameters
/// * `F` - The operation function type
/// * `T` - The return type of the operation
///
/// # Returns
/// Result containing the operation result or CommonError
///
/// # Behavior
/// - Retries operation up to max_attempts times
/// - Uses exponential backoff (doubles delay after each failure)
/// - Starts with 100ms delay
/// - Logs each attempt and failure
pub async fn recover_state_with_backoff<F, T>(
	operation:F,
	max_attempts:u32,
	operation_name:&str,
) -> Result<T, CommonError>
where
	F: Fn() -> Result<T, CommonError> + Send, {
	let mut attempt = 0;
	let mut delay_ms = 100;

	while attempt < max_attempts {
		match operation() {
			Ok(result) => return Ok(result),
			Err(error) => {
				attempt += 1;
				if attempt == max_attempts {
					return Err(error);
				}

				dev_log!("lifecycle", "warn: [RecoverState] Attempt {} failed for '{}': {}. Retrying in {}ms...",
					attempt, operation_name, error, delay_ms);

				tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

				// Apply exponential backoff by doubling the delay after each failure
				// to prevent overwhelming the system during recovery attempts.
				delay_ms *= 2;
			},
		}
	}

	Err(CommonError::Unknown {
		Description:format!(
			"Failed to recover state for '{}' after {} attempts",
			operation_name, max_attempts
		),
	})
}
