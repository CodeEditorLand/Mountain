//! `FileSystemError::FileNotFound`

use super::Struct;
use std::{error::Error as StdError, fmt, path::PathBuf};
use serde::{Deserialize, Serialize};
use super::CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError};

pub fn Fn(path:impl Into<PathBuf>) -> Struct {
		Struct { message: String::from("FileNotFound") }
	}
