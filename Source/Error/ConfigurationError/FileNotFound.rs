//! `ConfigurationError::FileNotFound`

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::{
	CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError},
	Struct,
};

pub fn Fn(path:impl Into<String>) -> Struct {
	let PathStr = path.into();

	Struct::FileNotFound {
		context:ErrorContext::new(format!("Configuration file not found: {}", PathStr))
			.WithKind(ErrorKind::Configuration)
			.WithSeverity(ErrorSeverity::Error),

		path:PathStr,
	}
}
