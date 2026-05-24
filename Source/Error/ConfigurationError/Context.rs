//! `ConfigurationError::Context`

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::{
	CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError},
	Struct,
};

pub fn Fn(This:&Struct) -> &ErrorContext {
	match self {
		ConfigurationError::KeyNotFound { context, .. } => context,

		ConfigurationError::InvalidValue { context, .. } => context,

		ConfigurationError::ValidationFailed { context, .. } => context,

		ConfigurationError::ParseError { context, .. } => context,

		ConfigurationError::FileNotFound { context, .. } => context,

		ConfigurationError::FileReadError { context, .. } => context,

		ConfigurationError::FileWriteError { context, .. } => context,

		ConfigurationError::CircularDependency { context, .. } => context,
	}
}
