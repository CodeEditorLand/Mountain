//! `ConfigurationService::New`

use super::Struct;
use std::collections::HashMap;

pub fn Fn() -> Struct { Self { config:HashMap::new() } }
