//! `ConfigurationService::Get`

use super::Struct;
use std::collections::HashMap;

pub fn Fn(This:&Struct, Key:&str) -> Option<&serde_json::Value> { This.config.get(Key) }
