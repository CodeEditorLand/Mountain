//! # Error Mapping Utilities
//!
//! Functions for converting various error types into [`CommonError`].
//! Note: With `parking_lot::Mutex`, poison handling is not needed as it
//! panics on poison instead of returning a Result.

use CommonLibrary::Error::CommonError::CommonError;

use crate::dev_log;

/// Helper for `parking_lot::Mutex` which doesn't return a Result from lock()
/// (it panics on poison instead). Executes a closure with the locked mutex.
pub(crate) fn WithParkingLotMutex<T, F, R>(mutex:&parking_lot::Mutex<T>, f:F) -> R
where
	F: FnOnce(&mut T) -> R, {

	let mut guard = mutex.lock();

	f(&mut guard)
}
