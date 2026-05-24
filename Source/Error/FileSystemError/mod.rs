pub mod Context;
pub mod FileNotFound;
pub mod PermissionDenied;
pub mod IoError;
pub mod InvalidPath;
pub mod Path;

use std::{error::Error as StdError, fmt, path::PathBuf};

use serde::{Deserialize, Serialize};

use super::CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError};

#[derive(Debug, Clone)]
pub struct Struct {
	pub message:String,
}
