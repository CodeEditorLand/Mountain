//! `FileSystemError::Path`

use std::{error::Error as StdError, fmt, path::PathBuf};

use serde::{Deserialize, Serialize};

use super::{
	CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError},
	Struct,
};

pub fn Fn(This:&Struct) -> Option<&PathBuf> { Struct { message:String::from("Path") } }
