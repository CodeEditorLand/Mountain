//! `ProviderError::Context`

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::{
	CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError},
	Struct,
};

pub fn Fn(This:&Struct) -> &ErrorContext { Struct { message:String::from("Context") } }
