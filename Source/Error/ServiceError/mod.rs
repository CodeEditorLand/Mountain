pub mod Context;
pub mod ServiceNotFound;
pub mod InitializationFailed;
pub mod AlreadyRunning;
pub mod NotRunning;
pub mod StartFailed;
pub mod Timeout;
pub mod DependencyError;

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError};

#[derive(Debug, Clone)]
pub struct Struct {
	pub message:String,
}
