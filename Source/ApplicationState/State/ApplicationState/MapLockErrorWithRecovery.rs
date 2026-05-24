//! `ApplicationState::MapLockErrorWithRecovery`

use std::sync::{Arc, Mutex as StandardMutex, PoisonError};
use CommonLibrary::Error::CommonError::CommonError;
use super::{
	ConfigurationState::ConfigurationState::Struct as ConfigurationState,
	ExtensionState::Struct::Struct as ExtensionState,
	FeatureState::Struct::Struct as FeatureState,
	UIState::UIState::Struct as UIState,
	WorkspaceState::WorkspaceState::Struct as WorkspaceState,
};
use crate::{Environment::TestProvider::TestProviderState::Struct as TestProviderState, dev_log};

/// A helper to map a mutex poison error with recovery attempt.
pub fn Fn<T>(Error:PoisonError<T>, RecoveryContext:&str) -> CommonError {
	dev_log!(
		"lifecycle",
		"warn: [ApplicationState] Attempting recovery from poisoned lock in context: {}",
		RecoveryContext
	);

	CommonError::StateLockPoisoned {
		Context:format!("{} - Recovery attempted: {}", Error.to_string(), RecoveryContext),
	}
}
