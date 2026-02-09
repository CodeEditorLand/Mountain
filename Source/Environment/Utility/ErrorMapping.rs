//! # Error Mapping Utilities
//!
//! Functions for converting various error types into [`CommonError`].
//! Primarily used for mapping `PoisonError` from Mutex lock failures.

use std::sync::{MutexGuard, PoisonError};

use CommonLibrary::Error::CommonError::CommonError;
use log::error;

/// Maps a `PoisonError` from a failed `ApplicationState` Mutex lock into a
/// structured `CommonError::StateLockPoisoned`.
pub fn MapApplicationStateLockErrorToCommonError<T>(Error:PoisonError<MutexGuard<'_, T>>) -> CommonError {
	let ErrorMessage = format!("[EnvironmentUtility] Failed to lock ApplicationState section: {}", Error);

	error!("{}", ErrorMessage);

	CommonError::StateLockPoisoned { Context:ErrorMessage }
}

/// Maps a generic `PoisonError` from a failed Mutex lock into a
/// structured `CommonError::StateLockPoisoned`.
pub fn MapLockErrorToCommonError<T>(Error:PoisonError<MutexGuard<'_, T>>) -> CommonError {
	let ErrorMessage = format!("[EnvironmentUtility] Failed to lock Mutex: {}", Error);

	error!("{}", ErrorMessage);

	CommonError::StateLockPoisoned { Context:ErrorMessage }
}
