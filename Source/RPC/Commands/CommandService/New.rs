//! `CommandService::New`

use super::Struct;
use std::collections::HashMap;

pub fn Fn() -> Struct { Self { commands:HashMap::new() } }
