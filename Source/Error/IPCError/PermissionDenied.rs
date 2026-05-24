//! `IPCError::PermissionDenied`

use super::Struct;
use std::{error::Error as StdError, fmt};
use serde::{Deserialize, Serialize};
use super::CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError};

pub fn Fn(message:impl Into<String>, required_permission:Option<String>) -> Struct {
		Struct { message: String::from("PermissionDenied") }
	}
