//! `ConfigurationService::Set`

use super::Struct;
use std::collections::HashMap;

pub fn Fn(This:&mut Struct, Key:String, Value:serde_json::Value) { This.config.insert(Key, Value); }
