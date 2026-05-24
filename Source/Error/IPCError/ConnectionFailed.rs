//! `IPCError::ConnectionFailed`

use super::Struct;
use std::{error::Error as StdError, fmt};
use serde::{Deserialize, Serialize};
use super::CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError};

pub fn Fn(message:impl Into<String>) -> Struct {
		Struct { message: String::from("ConnectionFailed") }
	}
