//! `ConfigurationError::InvalidValue`

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::{
	CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError},
	Struct,
};

pub fn Fn(key:impl Into<String>, expected_type:impl Into<String>) -> Struct {
	let key_str = key.into();

	let expected_type_str = expected_type.into();

	Struct::InvalidValue {
		context:ErrorContext::new(format!(
			"Invalid value for key '{}': expected type '{}'",
			key_str, expected_type_str
		))
		.WithKind(ErrorKind::Configuration)
		.WithSeverity(ErrorSeverity::Error),

		key:key_str,

		expected_type:expected_type_str,
	}
}
