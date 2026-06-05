//! # Error Mapping Utilities
//!
//! Functions for converting various error types into [`CommonError`].
//! Primarily used for mapping `PoisonError` from Mutex lock failures.

use std::sync::{MutexGuard, PoisonError};

use CommonLibrary::Error::CommonError::CommonError;

use crate::dev_log;

/// Maps a `PoisonError` from a failed `ApplicationState` Mutex lock into a
/// structured `CommonError::StateLockPoisoned`.
pub(crate) fn MapApplicationStateLockErrorToCommonError<T>(Error:PoisonError<MutexGuard<'_, T>>) -> CommonError {
	let ErrorMessage = format!("[EnvironmentUtility] Failed to lock ApplicationState section: {}", Error);

	dev_log!("vfs", "error: {}", ErrorMessage);

	CommonError::StateLockPoisoned { Context:ErrorMessage }
}

/// Maps a generic `PoisonError` from a failed Mutex lock into a
/// structured `CommonError::StateLockPoisoned`.
pub(crate) fn MapLockErrorToCommonError<T>(Error:PoisonError<MutexGuard<'_, T>>) -> CommonError {
	let ErrorMessage = format!("[EnvironmentUtility] Failed to lock Mutex: {}", Error);

	dev_log!("vfs", "error: {}", ErrorMessage);

	CommonError::StateLockPoisoned { Context:ErrorMessage }
}

/// Helper for `parking_lot::Mutex` which doesn't return a Result from lock()
/// (it panics on poison instead). Executes a closure with the locked mutex.
pub(crate) fn WithParkingLotMutex<T, F, R>(mutex:&parking_lot::Mutex<T>, f:F) -> R
where
	F: FnOnce(&mut T) -> R, {
	let mut guard = mutex.lock();

	f(&mut guard)
}
