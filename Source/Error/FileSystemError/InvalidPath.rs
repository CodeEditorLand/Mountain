//! `FileSystemError::InvalidPath`

use std::{error::Error as StdError, fmt, path::PathBuf};

use serde::{Deserialize, Serialize};

use super::{
	CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError},
	Struct,
};

pub fn Fn(path:impl Into<PathBuf>) -> Struct { Struct { message:String::from("InvalidPath") } }
