//! `ConfigurationError::CircularDependency`

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::{
	CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError},
	Struct,
};

pub fn Fn(keys:Vec<String>) -> Struct {
	Struct::CircularDependency {
		context:ErrorContext::new(format!("Circular dependency detected in configuration: {}", keys.join(" -> ")))
			.WithKind(ErrorKind::Configuration)
			.WithSeverity(ErrorSeverity::Critical),

		keys,
	}
}
