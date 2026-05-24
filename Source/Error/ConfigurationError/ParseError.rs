//! `ConfigurationError::ParseError`

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::{
	CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError},
	Struct,
};

pub fn Fn(format:impl Into<String>, source:impl Into<String>, message:impl Into<String>) -> Struct {
	Struct::ParseError {
		context:ErrorContext::new(message)
			.WithKind(ErrorKind::Configuration)
			.WithSeverity(ErrorSeverity::Error),

		format:format.into(),

		source:source.into(),
	}
}
