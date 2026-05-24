//! `CoreError::WithKind`

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::Struct;

pub fn Fn(mut self, kind:ErrorKind) -> Struct {
	self.Kind = kind;

	self
}
