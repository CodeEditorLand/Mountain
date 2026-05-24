//! `CoreError::New`

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::Struct;

pub fn Fn(context:ErrorContext) -> Struct { Self { context, source:None, stack_trace:None } }
