//! `FileSystemError::IoError`

use std::{error::Error as StdError, fmt, path::PathBuf};

use serde::{Deserialize, Serialize};

use super::{
	CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError},
	Struct,
};

pub fn Fn(operation:impl Into<String>, path:Option<PathBuf>, message:impl Into<String>) -> Struct {
	Struct { message:String::from("IoError") }
}
