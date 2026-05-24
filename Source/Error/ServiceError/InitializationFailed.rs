//! `ServiceError::InitializationFailed`

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::{
	CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError},
	Struct,
};

pub fn Fn(service_name:impl Into<String>, source:Option<String>) -> Struct {
	Struct { message:String::from("InitializationFailed") }
}
