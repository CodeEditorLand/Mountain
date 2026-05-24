//! `DefaultConfigurations::CollectDefaultConfigurations`

use CommonLibrary::Error::CommonError::CommonError;
use serde_json::{Map, Value};
use crate::{ApplicationState::Struct::ApplicationState::ApplicationState, Environment::Utility};

/// Merge default configuration values from all scanned extensions into one
/// flat `{key → defaultValue}
