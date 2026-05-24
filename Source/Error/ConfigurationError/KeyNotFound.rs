//! `ConfigurationError::KeyNotFound`

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::{
	CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError},
	Struct,
};

pub fn Fn(key:impl Into<String>, section:Option<String>) -> Struct {
	let Key = key.into();

	let Message = if let Some(section) = &section {
		format!("Configuration key '{}' not found in section '{}'", key, section)
	} else {
		format!("Configuration key '{}' not found", key)
	};

	Struct::KeyNotFound {
		context:ErrorContext::new(message)
			.WithKind(ErrorKind::Configuration)
			.WithSeverity(ErrorSeverity::Error),

		key,

		section,
	}
}
