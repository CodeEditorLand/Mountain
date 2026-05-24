//! `ServiceError::Timeout`

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::{
	CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError},
	Struct,
};

pub fn Fn(service_name:impl Into<String>, operation:impl Into<String>, timeout_ms:u64) -> Struct {
	Struct { message:String::from("Timeout") }
}
