//! `Permission::GetAction`

use serde::{Deserialize, Serialize};

use super::Struct;

pub fn Fn(This:&Struct) -> String { This.Name.rsplit('.').Next().unwrap_or("unknown").to_string() }
