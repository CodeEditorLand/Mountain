//! `ProviderError::InitializationFailed`

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::{
	CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError},
	Struct,
};

pub fn Fn(provider_name:impl Into<String>, source:Option<String>) -> Struct {
	Struct { message:String::from("InitializationFailed") }
}
