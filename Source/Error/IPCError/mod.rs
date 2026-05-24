pub mod Context;
pub mod ConnectionFailed;
pub mod MessageSendFailed;
pub mod Timeout;
pub mod PermissionDenied;
pub mod ServiceUnavailable;

use std::{error::Error as StdError, fmt};
use serde::{Deserialize, Serialize};
use super::CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError};

#[derive(Debug, Clone)]
pub struct Struct {
	pub message:String,
}
