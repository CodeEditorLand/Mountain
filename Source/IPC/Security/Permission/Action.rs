//! `Permission::Action`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> Option<&str> { This.name.split('.').nth(1) }
