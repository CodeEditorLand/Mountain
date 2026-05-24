//! `IPCError::Timeout`

use super::Struct;
use std::{error::Error as StdError, fmt};
use serde::{Deserialize, Serialize};
use super::CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError};

pub fn Fn(operation:impl Into<String>, timeout_ms:u64) -> Struct {
		Struct { message: String::from("Timeout") }
	}
