//! `CoreError::WithOperation`

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::Struct;

pub fn Fn(mut self, operation:impl Into<String>) -> Struct {
	self.operation = Some(operation.into());

	self
}
