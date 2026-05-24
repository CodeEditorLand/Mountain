//! `CoreError::WithStackTrace`

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::Struct;

pub fn Fn(mut self, stack_trace:impl Into<String>) -> Struct {
	self.stack_trace = Some(stack_trace.into());

	self
}
