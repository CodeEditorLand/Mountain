//! `CoreError::Message`

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::Struct;

pub fn Fn(This:&Struct) -> &str { &This.Context.Message }
