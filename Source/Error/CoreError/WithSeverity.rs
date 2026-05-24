//! `CoreError::WithSeverity`

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::Struct;

pub fn Fn(mut self, severity:ErrorSeverity) -> Struct {
	self.Severity = severity;

	self
}
