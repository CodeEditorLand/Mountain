//! `ConfigurationError::ValidationFailed`

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::{
	CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError},
	Struct,
};

pub fn Fn(errors:Vec<String>) -> Struct {
	Struct::ValidationFailed {
		context:ErrorContext::new(format!("Configuration validation failed with {} error(s)", errors.len()))
			.WithKind(ErrorKind::Configuration)
			.WithSeverity(ErrorSeverity::Error),

		errors,
	}
}
