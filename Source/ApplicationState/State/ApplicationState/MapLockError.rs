//! `ApplicationState::MapLockError`

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

/// A helper to map a mutex poison error into a CommonError.
pub fn Fn<T>(Error:PoisonError<T>) -> CommonError {
	CommonError::StateLockPoisoned { Context:Error.to_string() }
}
