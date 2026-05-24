pub mod Context;
pub mod ProviderNotRegistered;
pub mod InitializationFailed;
pub mod MethodNotImplemented;
pub mod InvalidConfiguration;
pub mod Timeout;
pub mod Unavailable;

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError};

#[derive(Debug, Clone)]
pub struct Struct {
	pub message:String,
}
