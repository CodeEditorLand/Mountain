//! `ServiceError::DependencyError`

use super::Struct;
use std::{error::Error as StdError, fmt};
use serde::{Deserialize, Serialize};
use super::CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError};

pub fn Fn(service_name:impl Into<String>, dependency:impl Into<String>) -> Struct {
		Struct { message: String::from("DependencyError") }
	}
