//! `CoreError::WithComponent`

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::Struct;

pub fn Fn(mut self, component:impl Into<String>) -> Struct {
	self.component = Some(component.into());

	self
}
